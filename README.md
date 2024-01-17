# `did:ethr` Linked Resources + `did:ethr` Anoncreds Method + Demo
This project implements the following:
* [DID Linked Resources for did:ethr](./did_ethr_linked_resources/README.md)
* [Anoncreds Method for did:ethr](./did_ethr_anoncreds/README.md)
* [Full-flow Demo of Anoncreds Method](./did_ethr_anoncreds_demo)

This project defines an extension on the `did:ethr` method to support creation and retrieval of resources which are verifiably controlled by the given `did:ethr` DID (_DID Linked Resources_).

DID Linked Resources can be used for many applications, however this project demonstrates how these resources can be used in an [Anoncreds](https://hyperledger.github.io/anoncreds-spec/) use case. This includes a full-flow Issuer/Holder/Verifier demo (with revocation support).

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
3. within `smart-contracts`: use hardhat to run a local ledger in a seperate terminal: `npx hardhat node`
4. within `smart-contracts`: use hardhat to deploy the `EthrDIDLinkedResourcesRegistry` & `EthereumDIDRegistry` contract to the local ledger: `npx hardhat run --network localhost scripts/deploy.ts`
   - Lookup value `Contract address` in the output. You need to provide in the next step as env variable.
5. within `did_ethr_anoncreds_demo`: run the demo!: `RESOURCES_REGISTRY_ADDRESS=<the_value_from_previous_step> cargo run`

## Demo with The Graph
As discussed in the [did:ethr DID Linked Resource documentation](./did_ethr_linked_resources/README.md), the DID Linked Resource resolver can be ran in "The Graph" mode. Running in this manner allows the resolver to retrieve resources more effectively, since it now relies on dedicated indexers.

Within [the subgraph directory](./example-subgraph/) is a subgraph project which can be used to index the `EthrDIDLinkedResourcesRegistry` smart contract.

When the subgraph is deployed, this demo can be ran in an alternative mode to utilize "The Graph" indexing.

## Local Graph Setup and Demo
1. complete steps 1-4 of [above](#run)
2. clone the [graph-node repo](https://github.com/graphprotocol/graph-node)
3. run the graph-node via docker compose (`cd docker && docker compose up`)
4. within the `example-subgraph`: 
    1. If your contract address for `EthrDIDLinkedResourcesRegistry` is different to the default, update the address in the [subgraph.yaml](./example-subgraph/subgraph.yaml)
    2. codegen `npm run codegen`
    3. create the local subgraph: `npm run create-local`
    4. deploy the local subgraph: `npm run deploy-local`
5. within the `did_ethr_anoncreds_demo`: run the demo with the graph feature enabled!: `RESOURCES_REGISTRY_ADDRESS=<the_value_from_previous_step> cargo run --features thegraph`


# Related
* Anoncreds method registry: https://hyperledger.github.io/anoncreds-methods-registry/
* `did:cheqd` DID Linked Resources: https://docs.cheqd.io/identity/credential-service/did-linked-resources