# Anoncreds Demo using `did:ethr` Linked Resources
TODO
# `did:ethr` Anoncreds Method (v0)
## Anoncreds Objects
The following section describes how [Anoncreds spec](https://hyperledger.github.io/anoncreds-spec/) objects are expected to be stored as `did:ethr` Linked Resources. As well as how they are resolved back into the spec object.

For all of these types, the `resourceUri` of the resource becomes ID for the resource used in Anoncreds transactions. 

For instance: 
```json
"schemaId": "did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266/resources/1423"
```

### Schema
#### Resource Content 
The data should be stored as **bytes** of the JSON serialized Schema object as described in the [anoncreds spec](https://hyperledger.github.io/anoncreds-spec/).
```json
{
  "issuerId": "did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
  "name": "Example schema",
  "version": "0.0.1",
  "attrNames": ["name", "age", "vmax"]
}
```

#### Resource Parameters
The resource parameters should be set accordingly when submitted:
* `resourceType`: `"anonCredsSchema"`
* `resourceName`: `"{name used in schema}"`
* `resourceVersion`: `"{version used in schema}"`
* `mediaType`: `"application/octet-stream"`

#### Resource into Object
To assemble the resource content & metadata back into the Schema (when resolving), JSON deserialize the content bytes into the [spec defined type](https://hyperledger.github.io/anoncreds-spec/).

### Credential Definition
#### Resource Content
The data should be stored as **bytes** of the JSON serialized Credential Definition object as described in the [anoncreds spec](https://hyperledger.github.io/anoncreds-spec/).
```json
{
  "issuerId": "did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
  "schemaId": "did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266/resources/1",
  "type": "CL",
  "tag": "Example Cred Def",
  "value": {
    "primary": {
      "n": "779...397",
      "r": {
        "birthdate": "294...298",
        // ...
        "name": "410...200",
      },
      "rctxt": "774...977",
      "s": "750..893",
      "z": "632...005"
    }
  }
}
```

#### Resource Parameters
The resource parameters should be set accordingly when submitted:
* `resourceType`: `"anonCredsCredDef"`
* `resourceName`: `"{tag used in credDef}"`
* `resourceVersion`: `""` (not needed)
* `mediaType`: `"application/octet-stream"`

#### Resource into Object
To assemble the resource content & metadata back into the Credential Definition (when resolving), JSON deserialize the content bytes into the [spec defined type](https://hyperledger.github.io/anoncreds-spec/).

### Revocation Registry Definition
#### Resource Content
The data should be stored as **bytes** of the JSON serialized Revocation Registry Definition object as described in the [anoncreds spec](https://hyperledger.github.io/anoncreds-spec/).
```json
{
  "issuerId": "did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
  "revocDefType": "CL_ACCUM",
  "credDefId": "did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266/resources/2",
  "tag": "RevReg1",
  "value": {
    "publicKeys": {
      "accumKey": {
        "z": "1 0BB...386"
      }
    },
    "maxCredNum": 666,
    "tailsLocation": "https://my.revocations.tails/tailsfile.txt",
    "tailsHash": "91zvq2cFmBZmHCcLqFyzv7bfehHH5rMhdAG5wTjqy2PE"
  }
}
```

#### Resource Parameters
The resource parameters should be set accordingly when submitted:
* `resourceType`: `"anonCredsRevocRegDef"`
* `resourceName`: `"{tag used in revRegDef}"`
* `resourceVersion`: `""` (not needed)
* `mediaType`: `"application/octet-stream"`

#### Resource into Object
To assemble the resource content & metadata back into the Revocation Registry Definition (when resolving), JSON deserialize the content bytes into the [spec defined type](https://hyperledger.github.io/anoncreds-spec/).

### Revocation Registry Status Lists
#### Resource Content
The data should be stored as the following **bytes** from the Revocation Registry Status List object described in the [anoncreds spec](https://hyperledger.github.io/anoncreds-spec/).
```
| accumulator (PointG2 bytes) | (128 bytes)
| statusListBits (see below)  | (remaining bytes)
```
`statusListBits` should be the revocation list as a bit array, encoded into bytes. Where a `1` bit indicates the index of a revoked credential for the given revocation registry definiton (and `0` == non-revoked). 

Note that encoding into bytes may result in padding of `0`s at the end of the array if the `maxCredNum` is not a multiple of 8. The resolved bit array should be truncated to `maxCredNum` length.

#### Resource Parameters
The resource parameters should be set accordingly when submitted:
* `resourceType`: `"anonCredsStatusList"`
* `resourceName`: `"{tag used in revRegDef}"`
* `resourceVersion`: `""` (not needed)
* `mediaType`: `"application/octet-stream"`

#### Resource into Object
To assemble the resource content & metadata back into the [spec defined](https://hyperledger.github.io/anoncreds-spec/) Revocation Registry Status List type (when resolving):
* deconstruct the resource content bytes as described above, resolving the `accumulator`'s `PointG2` and the `revocationList` (bit array)
* convert the resource metadata `created` timestamp into the epoch `timestamp`
* obtain the `revRegDefId` using prior context. If no prior context, then a query can be made for the associated revRegDef can be made using the status list resource's `resourceName` and `resourceType=anonCredsRevocRegDef`. `?resourceName={statuslist resourceName}&resourceType=anonCredsRevocRegDef`

## Querying Status Lists Versions
### At a point in time
One of the most common use cases in Anoncreds flows in resolving a revocation status list at some point in time. Provers do this when creating a non revocation proof (NRP) against some status list version (proving they were not revoked at a point in time). And Verifiers similarly do this when resolving a status list version which the Prover is presenting for.

A status list for a moment in time can be queried using the following DID Linked Resource query:
```
{issuer DID}?resourceType=anonCredsStatusList&resourceName={rev reg def tag}&versionTime={time as an XML datetime}
```
Example:
```
did:ethr:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266?resourceType=anonCredsStatusList&resourceName=universityDegreeRevReg1&versionTime=2023-05-10T18:00:00Z
```

### Most recent (current)
Additionally, the most recent (current) status list can be resolved via a query:
```
{issuer DID}?resourceType=anonCredsStatusList&resourceName={rev reg def tag}
```

### Iterating over updates
Once some instance of a status list has been resolved, the chain of updates to the status list can be chronologically (ascending or descending) be stepped thru by following the `previousVersionId` & `nextVersionId` fields of the resolved resource metadata.

For example if we resolve a status list with the metadata of:
```json
{
    // ...
    "previousVersionId": "123"
}
```

Then the resourceUri `{issuerDid}/resources/123` can be resolved to see the previous status list.

This may be useful when a Verifier requests a non-revocation **interval**, and the holder wishes to iterate over status list versions in that interval to find an instance where their credential is not revoked.