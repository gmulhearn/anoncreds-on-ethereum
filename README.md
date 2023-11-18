# Anoncreds on Ethereum Proof of Concept
Simple project demonstrating how a smart contract could be used as a VDR for anoncreds data (schemas, credential definitions, revocation registry definitions and revocation status list entries).

The `anoncreds-smart-contracts-js` directory contains a cookie cutter `hardhat` project for developing and deploying the `AnoncredsRegistry` & [EthereumDIDRegistry](https://github.com/uport-project/ethr-did-registry/blob/master/contracts/EthereumDIDRegistry.sol) smart contracts. Refer to the hardhat generated README for general usage of the hardhat project.

Whilst the `eth-anoncreds-rust-demo` directory contains a rust demo binary, which uses `ethers` to connect with the `AnoncredsRegistry` & `EthereumDIDRegistry` smart contract instances and use it as a VDR in standard anoncreds issuer/holder & verifier/prover flows.

# Implementation
To accomplish VDR functionality for immutably storing anoncreds assets, the `AnoncredsRegistry` smart contract has two main features:
* Storage and retrieval of arbitrary strings (resources), which are uniquely identified by a given `path` and authenticated `didIdentity` (Identity of the did:ethr DID. [See did:ethr spec](https://github.com/decentralized-identity/ethr-did-resolver/blob/master/doc/did-method-spec.md#relationship-to-erc1056))
* Storage and (somewhat optimised) retrieval of [Revocation Status Lists](https://hyperledger.github.io/anoncreds-spec/#term:revocation-status-list) for revocation registries.

## did:ethr Controller Authentication
Both types of resources mentioned above are stored against the identity address of a `did:ethr`. The write operations within `AnoncredsRegistry` require that the message signer (ethereum transaction signer) is coming from the _DID controller_ of the `did:ethr` for which the resource is being written for.

This is done by having `AnoncredsRegistry` look up the controller/owner of the did:ethr identity within the `EthereumDIDRegistry` via the contract's `identityOwner` method. For convenience, a smart contract modifier `onlyDidIdentityOwner` has been created to easily check this authentication.

## Arbitrary String Resources
Relatively simple in implementation, the smart contract stores resource data in the following map on the ledger:
```solidity
mapping(address => mapping(string => string)) immutableResourceByPathByDidIdentity;
```
I.e. meaning "Arbitrary strings are stored against String `path`s in a map, and these maps are stored against particular authenticated `didIdentity`s (ethereum address identity of the did:ethr DID)"

Smart contract setter and getters are then provided for this map:
```solidity
function createImmutableResource(address didIdentity, string memory path, string memory content)

function getImmutableResource(address didIdentity, string memory path) public view returns (string memory)
```

### DID Resource Identifiers

We then create "DID Resource Identifiers" which can be constructed and deconstructed on the client side to represent pointers to these immutable artifacts. These resources are stored and authenticated against the controller of the did:ethr identity which is uploading them.

For example:
```
did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266/schema/c9f56e91-8408-40b9-b214-c3ebcb2f71d9
```
This is a resource identifier which refers to an immutable resource in this smart contract, where the `didIdentity` is the ethereum address identity associated with this DID (`0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266`), and the resource path is everything appended to the path (`schema/c9f56e91-8408-40b9-b214-c3ebcb2f71d9`).

The above example is an identifier being used to store an anoncreds schema as a JSON string resource, however it can be used for any string data.

## Revocation Status Lists
### Problem
Anoncreds artifacts such as `Schemas`, `Cred Defs` and `Revocation Registry Defs` can be trivially stored as resources found via [DID Resource Identifiers](#did-resource-identifiers), however [Revocation Status Lists](https://hyperledger.github.io/anoncreds-spec/#term:revocation-status-list) are an artifact that need special handling for indexing optimisation reasons. 

This is particularly due to the nature of how these artifacts are fetched by anoncred agents. Schemas, Cred Defs and Rev Reg Defs are all typically fetched by agents via an already known ID (in our case we are using [DID Resource Identifiers](#did-resource-identifiers) as the IDs). These are known as they are passed around in data when engaging in protocols (issue-credential, present-proof).

However, revocation status list entries are typically retrieved in a more _dynamic_ fashion. For instance, when a Holder receives a proof request which requests a NRP (non revocation proof) for the time interval of `15-20`, it is their duty to scan the ledger (and/or local cache) to find a moment in time between `15-20`* where the revocation status list shows that their credential is **not** revoked, this status list would then be used by the holder to create a NRP with anoncreds libraries.

_*note: if there is no entry because this range, then the holder will use the closest entry made before the time `15`_.

In general, we want to optimise the revocation status lists such that; given a revocation registry ID, the revocation status list entries within a particular timestamp range can be found as quickly as possible.

### Approach
As such, we should try optimise these ledger look ups in our smart contract such that the consumer does not have to scan the entire list of revocation status list data every time. The semi-optimised approach taken in this demo is as follows:

The smart contract stores 2 maps of data on the ledger:
```rust
struct RevocationStatusList {
    string revocationList;
    string currentAccumulator;
}

mapping(address => mapping(string => RevocationStatusList[])) revStatusListsByRevRegIdByDidIdentity;

mapping(address => mapping(string => uint32[])) revStatusUpdateTimestampsByRevRegIdByDidIdentity;
```

The first map, `revStatusListsByRevRegIdByDidIdentity`, stores the actual `RevocationStatusList` entries uniquely against the ID of the revocation registry AND the authenticated DID identity. The list of `RevocationStatusList`s per revocation registry ID is ordered in chronological order. `RevocationStatusList` items are relatively large due to the data it holds*.

_*There is room for optimisation here, particularly serializing to string is sub-optimal for the data it stores._

The second map, `revStatusUpdateTimestampsByRevRegIdByDidIdentity`, acts as metadata for quicker lookups into the first map based on the desired timestamp. This map stores a list of epoch timestamp (`u32`) entries per revocation registry ID. The timestamp entries in these lists are 1-to-1 with the first map, i.e. index `[i]` in the timestamp list will have a value which is the timestamp associated with the index `[i]` entry in the `RevocationStatusList[]` list from the `revStatusListsByRevRegIdByDidIdentity` map.

```js
let timestamp = revStatusUpdateTimestampsByRevRegIdByDidIdentity["0xABC"]["revreg1"][i]

let statusList = revStatusListsByRevRegIdByDidIdentity["0xABC"]["revreg1"][i]

// here, `timestamp` is the epoch timestamp for which the `statusList` entry was made on the ledger
```

Given the relatively smaller size of lists within `revStatusUpdateTimestampsByRevRegIdByDidIdentity`, the idea is that consumers can retrieve the full list of timestamps for a given `revocationRegistryId`, then they can locally scan thru that list of timestamps to find the `index` of a timestamp which is near a desired timestamp they had in mind (e.g. a non-revoked interval for a proof request). Then this `index` can be used to get the `RevocationStatusList` stored on the ledger at this `index` for the given `revocationRegistryId`. 

This optimisation is especially neccessary, as fetching the full list of `RevocationStatusList[]`s from the `revStatusListsByRevRegIdByDidIdentity` may be too large of a transaction for some ethereum ledgers/RPCs to handle.

An example of how this approach is used can be seen [here](./eth-anoncreds-rust-demo/src/anoncreds_eth_registry.rs#L219).

#### Disclaimer
_The 'optimisation' approach taken by this demo is far from perfect. It is just a slight optimisation done to draw attention to the idea that revocation status lists need optimisation considerations, particularly for `timestamp` lookups._

# Demo
The demo within the Rust crate walks thru the following:
* Creating DID Resources for a did:ethr, and testing auth when the did:ethr controller changes
* Creating anoncred artifacts (schema, cred def, revocation registry def) and writing them to the registry
* Receiving and storing a credential which uses these anoncreds artifacts (i.e. demonstrating how they are read from the registry)
* Creating and verifying proof presentations of credentials which use these anoncreds artifacts
* Make revocation registry entries to revoke and un-revoke credentials
* Creating and verifying proof presentations with NRPs, including scanning the registry to find appropriate revocation status list entries to use for NRPs.

## Run

To setup and run the demo:
1. create your `.env` file in the root of this project. Using `.env.example` as an example.
2. within `anoncreds-smart-contracts-js`: `npm install`
3. within `anoncreds-smart-contracts-js`: use hardhat to run a local ledger in a seperate terminal: `npx hardhat node`
4. within `anoncreds-smart-contracts-js`: use hardhat to deploy the `AnoncredsRegistry` & `EthereumDIDRegistry` contract to the local ledger: `npx hardhat run --network localhost scripts/deploy.ts`
5. copy the deployed address into the `REGISTRY_WETH_ADDRESS` const of the [anoncreds_eth_registry.rs file](/eth-anoncreds-rust-demo/src/anoncreds_eth_registry.rs)
6. within `eth-anoncreds-rust-demo`: run the demo!: `cargo run`