# DID Linked Resources for `did:ethr`
This crate contains an implementation of [DID Linked Resource](https://wiki.trustoverip.org/display/HOME/DID-Linked+Resources+Specification) for the [did:ethr DID Method](https://github.com/decentralized-identity/ethr-did-resolver/blob/master/doc/did-method-spec.md).

This allows immutable resources to be stored on Ethereum ledgers (VDR) by `did:ethr` DIDs, and retrieved/queried according to the [spec](https://wiki.trustoverip.org/display/HOME/DID-Linked+Resources+Specification). Resources are tied to the `did:ethr` subject DID, as only the **DID controller** is authorized to submit resources.

This crate provides a `Resolver` interface for querying/resolving resources, and a `Registrar` interface for submitting a resource.

# Examples
An example of how the resolver and registrar are used can be seen in the [test demo](./src/lib.rs).

# Resolver Modes
The `Resolver` is notable implemented with 2 modes:
* **Pure Ethereum**
* **The Graph**
## Pure Ethereum
The default implementation of the resolver uses pure Ethereum APIs to fetch data from an Ethereum RPC. This implementation does not rely on any other means of ledger indexing, and works with any compliant Ethereum RPC. 

However due to the nature of Ethereum RPC APIs, indexing options are relatively limited. Notably Ethereum events can only index a maximum of 3 parameters. So indexing all resource parameters (e.g. `versionId`) for the sake of supporting all query variations is not possible.

## The Graph
To overcome the limitations of indexing possibilities using pure Ethereum APIs, an external Ethereum indexer can be used. A commonly used feature-rich indexer for dApps is [The Graph](https://thegraph.com/). The graph allows indexing on all event fields of smart contract events, meaning parameters non indexed in pure Ethereum APIs (e.g. `versionId`) can now be done. This also includes compartive indexing (e.g. `blockTimestamp >= x`).

However the Graph comes with it's own drawbacks. Most notable, reliance on 3rd parties for indexing.

# Spec Features
Aiming to align with the [spec](https://wiki.trustoverip.org/display/HOME/DID-Linked+Resources+Specification) as close as possible, the following features are currently supported:
* ✅ Submitting a resource to the ledger with the full set of parameters described in the spec
* ✅ Control over resources only permitted by controller of the DID Document
* ✅ Resolving full resource metadata (all `Resource Parameter` spec fields) & content
* ✅ Query for an exact `resourceUri` (e.g. `did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266/resources/3054`)
* ✅ Query for resource via `resourceName`, `resourceType` & `versionTime` (fetching a resource at a point in time)
* ✅ Query for _latest_ resource via `resourceName` & `resourceType`

Not currently supported features include:
* ❌ Control over resource permitted by non-controller `verificationMethod`s of the DID
* ❌ DID Document referencing associated resource via linked resource metadata
* ❌ Query with just `resourceName` or `resourceType` parameters
* ❌ Query with the following parameters: `resourceVersionId`/`versionId`, `linkedResource`, `resourceMetadata`, `latestResourceVersion`, `allResourceVersions`

# Other Features
* ✅ Configurable ledger
* 🚧 **Needs research:** Official integration with `did:ethr` OR creation of proxy DID method (e.g. `did:ethrplus`)
* 🚧 **Needs research:** Investigate IPFS for storage of content (only metadata on chain)

