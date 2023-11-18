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

    /// where revStatusUpdateTimestamps[i] == the timestamp of revStatusLists[i] below
    /// note that by issuer (address =>) is only needed for security purposes.
    mapping(address => mapping(string => uint32[])) private revStatusUpdateTimestampsByRevRegIdByDidIdentity;

    /// storage of revocation status lists, by the revocation registry ID, by the issuer.
    /// note that by issuer (address =>) is only needed for security purposes
    mapping(address => mapping(string => RevocationStatusList[])) private revStatusListsByRevRegIdByDidRegistry;
    
    /// simplified revocation status list. Containing only the data we care about, 
    /// the rest can be constructed by the client with other metadata.
    struct RevocationStatusList {
        string revocationList; // serialized bitvec (RON notation i believe)
        string currentAccumulator;
    }

    modifier onlyDidIdentityOwner(address identity) {
        address actor = msg.sender;
        require (actor == didRegistry.identityOwner(identity), "bad_actor");
        _;
    }
    event NewResource(address didIdentity, string path);
    event NewRevRegStatusUpdate(string revocationRegistryId, uint indexInStatusList, uint32 timestamp);

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
        emit NewResource(didIdentity, path);
    }

    /// Get the [content] of an immutable resource in this registry, identified by it's [path] and [didIdentity].
    function getImmutableResource(address didIdentity, string memory path) public view returns (string memory) {
        return immutableResourceByPathByDidIdentity[didIdentity][path];
    }

    /// Stores a new [statusList] within the list of status lists stored for the given [revocationRegistryId] (and [didIdentity]).
    ///
    /// Emits an event, [NewRevRegStatusUpdate], which contains the registry-determined timestamp for the statusList
    /// entry.
    function addRevocationRegistryStatusUpdate(address didIdentity, string memory revocationRegistryId, RevocationStatusList memory statusList) public onlyDidIdentityOwner(didIdentity) {
        uint32 timestamp = uint32(block.timestamp);

        revStatusUpdateTimestampsByRevRegIdByDidIdentity[didIdentity][revocationRegistryId].push(timestamp);
        revStatusListsByRevRegIdByDidRegistry[didIdentity][revocationRegistryId].push(statusList);

        uint newListLength = revStatusListsByRevRegIdByDidRegistry[didIdentity][revocationRegistryId].length;
        uint indexOfNewEntry = newListLength - 1;

        emit NewRevRegStatusUpdate(revocationRegistryId, indexOfNewEntry, timestamp);
    }

    /// Return the list of timestamps of revocation status list update that have been made for the given
    /// [revocationRegistryId] and [didIdentity].
    /// This list will naturally be chronologically sorted.
    /// The indexes in this list are 1-to-1 with the full status list list. For instance, index "5" in this list
    /// may contain a timestamp like "1697948227", this indicates that the status list at index "5" has a timestamp
    /// of "1697948227".
    /// 
    /// The intention is that the data size of this list will be smaller than the entire list of revocation
    /// status list entries. So a consumer looking for a revocation status list entry near a certain timestamp
    /// can retrieve this list of timestamps, then find the index of their desired timestamp, then look up that 
    /// index to get the full [RevocationStatusList] details via [getRevocationRegistryUpdateAtIndex].
    /// 
    /// Consumers may additionally wish to cache this list to avoid unneccessary future look ups.
    function getRevocationRegistryUpdateTimestamps(address didIdentity, string memory revocationRegistryId) public view returns (uint32[] memory) {
        return revStatusUpdateTimestampsByRevRegIdByDidIdentity[didIdentity][revocationRegistryId];
    }

    /// Returns the full [RevocationStatusList] entry information of a particular revocation registry at a particular index.
    ///
    /// consumers are intended to use [getRevocationRegistryUpdateTimestamps] to know exactly what [index] they are looking for.
    function getRevocationRegistryUpdateAtIndex(address didIdentity, string memory revocationRegistryId, uint index) public view returns (RevocationStatusList memory) {
        return revStatusListsByRevRegIdByDidRegistry[didIdentity][revocationRegistryId][index];
    }
}
