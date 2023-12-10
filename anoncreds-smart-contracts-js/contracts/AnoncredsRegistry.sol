// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./EthereumDIDRegistry.sol";

/// Contract for storing and retrieving immutable resources (e.g. anoncreds assets)
/// uploaded by an authenticated signer [address].
/// Also allow for storing and retrieving revocation status list updates, and some 
/// mechanisms for efficient lookups of revocation status lists by [timestamp].
contract AnoncredsRegistry {

    EthereumDIDRegistry public didRegistry;

    /// storage of string blobs, which are immutable once uploaded, and identified by its path + DID Identity
    mapping(address => mapping(string => string)) private immutableResourceByPathByDidIdentity;

    /// storage of current revocation status list, by the revocation registry ID, by the issuer.
    /// note that by issuer (address =>) is only needed for security purposes
    mapping(address => mapping(string => RevocationStatusList)) private statusListByRevRegIdByDidIdentity;

    /// storage of status list update metadatas, by the revocation registry ID, by the issuer.
    /// this exists for indexing purposes; clients can use this list to find an update blocknumber/timestamp
    /// nearest to their desired timestamp, and then perform a StatusListUpdateEvent event query
    /// to find the status list at that timestamp.
    /// note that by issuer (address =>) is only needed for security purposes
    mapping(address => mapping(string => RevocationStatusListUpdateMetadata[])) private statusListUpdateMetadataByRevRegIdByDidIdentity;
    
    struct UpdateRevocationStatusListInput {
        string revocationList; // serialized bitvec (RON notation i believe) - TODO optimise
        string currentAccumulator;
    }

    /// simplified revocation status list. Containing only the data we care about, 
    /// the rest can be constructed by the client with other metadata.
    struct RevocationStatusList {
        string revocationList; // serialized bitvec (RON notation i believe) - TODO optimise
        string currentAccumulator;
        RevocationStatusListUpdateMetadata metadata;
        RevocationStatusListUpdateMetadata previousMetadata;
    }

    struct RevocationStatusListUpdateMetadata {
        uint32 blockTimestamp;
        // TODO - is uint32 fine for block number? we want to go as low as feasible
        uint32 blockNumber;
    }

    modifier onlyDidIdentityOwner(address identity) {
        address actor = msg.sender;
        require (actor == didRegistry.identityOwner(identity), "bad_actor");
        _;
    }
    event NewResourceEvent(address didIdentity, string path);
    event StatusListUpdateEvent(string indexed indexedRevocationRegistryId, string revocationRegistryId, RevocationStatusList statusList);

    constructor(address didRegistryAddress) {
        didRegistry = EthereumDIDRegistry(didRegistryAddress);
    }

    function doesImmutableResourceExist(address didIdentity, string memory path) private view returns (bool) {
        string memory resource = immutableResourceByPathByDidIdentity[didIdentity][path];
        return bytes(resource).length != 0;
    }

    /// Store [content] as an immutable resource in this registry. Where [content] is uniquely identified
    /// by the [path] and the DID identity.
    /// Note that since this is immutable data, repeated [path]s can only be used once per given DID Identity.
    function createImmutableResource(address didIdentity, string memory path, string memory content) public onlyDidIdentityOwner(didIdentity) {
        require(!doesImmutableResourceExist(didIdentity, path), "Resource already created with this Path and DID");
        immutableResourceByPathByDidIdentity[didIdentity][path] = content;
        emit NewResourceEvent(didIdentity, path);
    }

    /// Get the [content] of an immutable resource in this registry, identified by it's [path] and [didIdentity].
    function getImmutableResource(address didIdentity, string memory path) public view returns (string memory) {
        return immutableResourceByPathByDidIdentity[didIdentity][path];
    }

    /// Stores an updated [statusList] for the given [revocationRegistryId] (and [didIdentity]).
    ///
    /// Emits an event, [StatusListUpdateEvent], which contains the registry-determined timestamp for the statusList
    /// entry.
    function updateRevocationRegistryStatusList(address didIdentity, string memory revocationRegistryId, UpdateRevocationStatusListInput memory statusListInput) public onlyDidIdentityOwner(didIdentity) {
        uint32 blockTimestamp = uint32(block.timestamp);
        uint32 blockNumber = uint32(block.number);

        RevocationStatusListUpdateMetadata memory previousMetadata = statusListByRevRegIdByDidIdentity[didIdentity][revocationRegistryId].metadata;

        // Enforce no simultaneous updates
        require(blockNumber > previousMetadata.blockNumber);
        require(blockTimestamp > previousMetadata.blockTimestamp);

        RevocationStatusListUpdateMetadata memory metadata = RevocationStatusListUpdateMetadata(
            blockTimestamp,
            blockNumber
        );

        RevocationStatusList memory statusList = RevocationStatusList(
            statusListInput.revocationList,
            statusListInput.currentAccumulator,
            metadata,
            previousMetadata
        );

        // store an update metadata for client-side indexing purposes
        statusListUpdateMetadataByRevRegIdByDidIdentity[didIdentity][revocationRegistryId].push(metadata);

        // set the new list
        statusListByRevRegIdByDidIdentity[didIdentity][revocationRegistryId] = statusList;
        emit StatusListUpdateEvent(revocationRegistryId, revocationRegistryId, statusList);
    }

    function getCurrentRevocationRegistryStatusList(address didIdentity, string memory revocationRegistryId) public view returns (RevocationStatusList memory) {
        return statusListByRevRegIdByDidIdentity[didIdentity][revocationRegistryId];
    }

    /// Return the list of timestamps of revocation status list update that have been made for the given
    /// [revocationRegistryId] and [didIdentity].
    /// This list will naturally be chronologically sorted.
    /// 
    /// The intention is that the data size of this list will be smaller than the entire list of revocation
    /// status list entries. So a consumer looking for a revocation status list entry near a certain timestamp
    /// can retrieve this list of timestamps, then find the index of their desired timestamp, then look up that 
    /// index to get the full [RevocationStatusList] details via StatusListUpdateEvent event filtering.
    /// 
    /// Consumers may additionally wish to cache this list to avoid unneccessary future look ups.
    function getRevocationRegistryStatusListUpdatesMetadata(address didIdentity, string memory revocationRegistryId) public view returns (RevocationStatusListUpdateMetadata[] memory) {
        return statusListUpdateMetadataByRevRegIdByDidIdentity[didIdentity][revocationRegistryId];
    }
}
