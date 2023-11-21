# did:ethr resolver demo

This demo assumes you have already executed `eth-anoncreds-rust-demo` which
creates registers `did:ethr` in DID registry.

This demo resolves the created Did Document for the registered DID using JS `did:ethr`
resolver implementation based on:
- [`did-resolver`](https://github.com/decentralized-identity/did-resolver)
- [`ethr-did-resolver`](https://github.com/decentralized-identity/ethr-did-resolver)

# Instructions
1. Install dependencies
```sh
npm install
```

2. Run demo 
```
DID_REGISTRY_ADDRESS=<your_registry_address> DID=<your_did> npm run demo
```

If no env variables are provided, following defaults are used:
- DID_REGISTRY_ADDRESS: `0x5fbdb2315678afecb367f032d93f642f64180aa3`
- DID: `did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266`

The demo should output resolved DID Document, such as:
```json

{
  "didDocumentMetadata": {
    "versionId": "2",
    "updated": "2023-11-21T21:01:40Z"
  },
  "didResolutionMetadata": {
    "contentType": "application/did+ld+json"
  },
  "didDocument": {
    "id": "did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
    "verificationMethod": [
      {
        "id": "did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266#controller",
        "type": "EcdsaSecp256k1RecoveryMethod2020",
        "controller": "did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
        "blockchainAccountId": "eip155:31337:0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
      }
    ],
    "authentication": [
      "did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266#controller"
    ],
    "assertionMethod": [
      "did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266#controller"
    ],
    "@context": [
      "https://www.w3.org/ns/did/v1",
      "https://w3id.org/security/suites/secp256k1recovery-2020/v2",
      "https://w3id.org/security/v3-unstable"
    ]
  }
}
```