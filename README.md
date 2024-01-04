# Anoncreds on Ethereum Proof of Concept
Simple project demonstrating how a smart contract could be used as a VDR for anoncreds data (schemas, credential definitions, revocation registry definitions and revocation status lists).

The `anoncreds-smart-contracts-js` directory contains a cookie cutter `hardhat` project for developing and deploying the `AnoncredsRegistry` & [EthereumDIDRegistry](https://github.com/uport-project/ethr-did-registry/blob/master/contracts/EthereumDIDRegistry.sol) smart contracts. Refer to the hardhat generated README for general usage of the hardhat project.

Whilst the `eth-anoncreds-rust-demo` directory contains a rust demo binary, which uses `ethers` to connect with the `AnoncredsRegistry` & `EthereumDIDRegistry` smart contract instances and use it as a VDR in standard anoncreds issuer/holder & verifier/prover flows.

# Implementation
To accomplish VDR functionality for immutably storing anoncreds assets, the `AnoncredsRegistry` smart contract has two main features:
* Storage and retrieval of arbitrary strings (resources), which are uniquely identified by a given `path` and authenticated `didIdentity` (Identity of the did:ethr DID. [See did:ethr spec](https://github.com/decentralized-identity/ethr-did-resolver/blob/master/doc/did-method-spec.md#relationship-to-erc1056))
* Storage and optimised retrieval of current (and historical) [Revocation Status Lists](https://hyperledger.github.io/anoncreds-spec/#term:revocation-status-list) for revocation registries.

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
Anoncreds artifacts such as `Schemas`, `Cred Defs` and `Revocation Registry Defs` can be trivially stored as resources found via [DID Resource Identifiers](#did-resource-identifiers). These resources never change once uploaded, they are _purely immutable_ however [Revocation Status Lists](https://hyperledger.github.io/anoncreds-spec/#term:revocation-status-list) are an artifact that are _historically immutable_ (the current value can be changed, but the history of the status list is preserved immutably). These artifacts need special handling for historical lookup optimisation reasons.

This is particularly due to the nature of how these artifacts are fetched by anoncred agents. Schemas, Cred Defs and Rev Reg Defs are all typically fetched by agents via an already known ID (in our case we are using [DID Resource Identifiers](#did-resource-identifiers) as the IDs). These are known as they are passed around in data when engaging in protocols (issue-credential, present-proof).

However, revocation status lists are typically retrieved in a more _dynamic_ fashion. For instance, when a Holder receives a proof request which requests a NRP (non revocation proof) for the time interval of `15-20`, it is their duty to scan the ledger (and/or local cache) to find a moment in time between `15-20`* where the historical revocation status list shows that their credential is **not** revoked, this status list would then be used by the holder to create a NRP with anoncreds libraries.

_*note: if there is no entry between this range, then the holder will use the closest entry made before the time `15`_.

In general, we want to optimise the revocation status lists such that; given a revocation registry ID, the history of a revocation status list within a particular timestamp range can be found as quickly as possible.

### Approach
As such, we should try optimise these ledger look ups in our smart contract such that the smart contract does not have to store the entire history of status lists, and the consumer does not have to query the ledger many times. The semi-optimised approach taken in this demo is as follows:

The smart contract stores 2 maps of data on the ledger:
```rust
struct RevocationStatusList {
    string revocationList;
    string currentAccumulator;
    ...
}

struct RevocationStatusListUpdateMetadata {
    uint32 blockTimestamp;
    uint32 blockNumber;
}

mapping(address => mapping(string => RevocationStatusList)) statusListByRevRegIdByDidIdentity;

mapping(address => mapping(string => RevocationStatusListUpdateMetadata)) statusListUpdateMetadataByRevRegIdByDidIdentity;
```

The smart contract also emits an event whenever an update is made:
```rust
event StatusListUpdateEvent(
    string indexed indexedRevocationRegistryId, 
    string revocationRegistryId, 
    RevocationStatusList statusList, 
    uint32 timestamp
);
```

The first map, `statusListByRevRegIdByDidIdentity`, stores the actual current `RevocationStatusList` uniquely against the ID of the revocation registry AND the authenticated DID identity. `RevocationStatusList` items are relatively large due to the data it holds*.

_*There is more room for optimisation here, particularly serializing to string is sub-optimal for the data it stores._

This map can be used to lookup the CURRENT statusList for a given revocation registry.

The second map, `statusListUpdateMetadataByRevRegIdByDidIdentity`, acts as index for determining the blocknumber of a status list update nearest to a desired timestamp. This map stores a list of the timestamps and blocknumbers for each status list update.

Given the relatively smaller size of lists within `statusListUpdateMetadataByRevRegIdByDidIdentity`, the idea is that consumers can retrieve the full list of timestamps for a given `revocationRegistryId`, then they can locally scan thru that list of metadata to find the `blockNumber` of a statuslist update which is near a desired timestamp they had in mind (e.g. a non-revoked interval for a proof request). 

Then this `blockNumber` can be used to get the `StatusListUpdateEvent` event emitted at the exact `blockNumber` for the given `revocationRegistryId`. The Ethereum API is designed/optimised for these sorts of indexed event lookups*.

_*The Ethereum API does not support indexed event querying by timestamp range, only by blocknumber range. This limitation is why we need to use the metadata to translate timestamp -> blocknumber._

This optimisation is especially neccessary, as storing and fetching the full historical list of `RevocationStatusList`s from the smart contract may be too large of a transaction for some ethereum ledgers/RPCs to handle.

An example of how this approach is used can be seen [here](./eth-anoncreds-rust-demo/src/anoncreds_eth_registry.rs#L219).

#### Disclaimer
_The 'optimisation' approach taken by this demo is not perfect. It is just a slight optimisation done to draw attention to the idea that revocation status lists need optimisation considerations, particularly for `timestamp` lookups._

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
2. `npm install`
3. within `anoncreds-smart-contracts-js`: use hardhat to run a local ledger in a seperate terminal: `npx hardhat node`
4. within `anoncreds-smart-contracts-js`: use hardhat to deploy the `AnoncredsRegistry` & `EthereumDIDRegistry` contract to the local ledger: `npx hardhat run --network localhost scripts/deploy.ts`
   - Lookup value `Contract address` in the output. You need to provide in the next step as env variable.
5. within `eth-anoncreds-rust-demo`: run the demo!: `ANONCRED_REGISTRY_ADDRESS=<the_value_from_previous_step> cargo run`

## Integration with The Graph
As mentioned above, a common use case for holders when creating NRPs is to find `StatusListUpdateEvent` events which occur between a range of time, or as to close a timestamp as possible without being later. The native Ethereum API does not support that type of event filtering, which is what lead to the [approach discussed above](#approach). However, an alternative to that, is to use Ethereum indexing infrastructure, such as [The Graph](https://thegraph.com/), which allows for these queries to be performed.

Within [the subgraph directory](./anoncreds-registry-subgraph/) is a subgraph project which can be used to index the `AnoncredsRegistry` smart contract.

When the subgraph is deployed, it can be queried with graphql to "get `StatusListUpdateEvent`s between a range of time" and much more. This is an alternative demo flow for this project.

## Local Graph Setup and Demo
1. complete steps 1-4 of [above](#run)
2. clone the [graph-node repo](https://github.com/graphprotocol/graph-node)
3. run the graph-node via docker compose (`cd docker && docker compose up`)
4. within the `anoncreds-registry-subgraph`: 
    1. If your contract address for `AnoncredsRegistry` is different to the default, update the address in the [subgraph.yaml](./anoncreds-registry-subgraph/subgraph.yaml)
    2. codegen `npm run codegen`
    3. create the local subgraph: `npm run create-local`
    4. deploy the local subgraph: `npm run deploy-local`
5. within the `eth-anoncreds-rust-demo`: run the demo with the graph feature enabled!: `ANONCRED_REGISTRY_ADDRESS=<the_value_from_previous_step> cargo run --features thegraph`
