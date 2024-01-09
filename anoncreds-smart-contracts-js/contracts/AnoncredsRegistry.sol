// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./EthereumDIDRegistry.sol";

/// Contract for storing and retrieving immutable resources (e.g. anoncreds assets)
/// uploaded by an authenticated signer [address].
/// TODO
contract AnoncredsRegistry {

    EthereumDIDRegistry public didRegistry;

    /// storage of string blobs, which are immutable once uploaded, and identified by its path + DID Identity
    mapping(address => mapping(string => string)) private immutableResourceByPathByDidIdentity;

    mapping(address => mapping(string => MutableResource)) private mutableResourceByPathByDidIdentity;

    mapping(address => mapping(string => MutableResourceUpdateMetadata[])) private mutableResourceUpdateMetadataByPathByDidIdentity;

    struct MutableResource {
        bytes content;
        MutableResourceUpdateMetadata metadata;
        MutableResourceUpdateMetadata previousMetadata;
    }

    struct MutableResourceUpdateMetadata {
        uint40 blockTimestamp;
        uint64 blockNumber;
    }

    modifier onlyDidIdentityOwner(address identity) {
        address actor = msg.sender;
        require (actor == didRegistry.identityOwner(identity), "bad_actor");
        _;
    }
    event NewResourceEvent(address didIdentity, string path);
    event MutableResourceUpdateEvent(address indexed didIdentity, string indexed indexedPath, string path, MutableResource resource);

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

    /// Update the mutable resource at the given [path] and [didIdentity] with the given [content].
    /// 
    function updateMutableResource(address didIdentity, string memory path, bytes memory content) public onlyDidIdentityOwner(didIdentity) {
        uint32 blockTimestamp = uint32(block.timestamp);
        uint32 blockNumber = uint32(block.number);

        MutableResource memory previousResource = mutableResourceByPathByDidIdentity[didIdentity][path];

        // Enforce no simultaneous updates
        require(blockNumber > previousResource.metadata.blockNumber);
        require(blockTimestamp > previousResource.metadata.blockTimestamp);

        MutableResourceUpdateMetadata memory metadata = MutableResourceUpdateMetadata(
            blockTimestamp,
            blockNumber
        );

        MutableResource memory resource = MutableResource(
            content,
            metadata,
            previousResource.metadata
        );

        // store an update metadata for client-side indexing purposes
        mutableResourceUpdateMetadataByPathByDidIdentity[didIdentity][path].push(metadata);

        // set the new resource
        mutableResourceByPathByDidIdentity[didIdentity][path] = resource;
        emit MutableResourceUpdateEvent(didIdentity, path, path, resource);
    }

    function getCurrentMutableResource(address didIdentity, string memory path) public view returns (MutableResource memory) {
        return mutableResourceByPathByDidIdentity[didIdentity][path];
    }

    function getMutableResourceUpdatesMetadata(address didIdentity, string memory path) public view returns (MutableResourceUpdateMetadata[] memory) {
        return mutableResourceUpdateMetadataByPathByDidIdentity[didIdentity][path];
    }
}
