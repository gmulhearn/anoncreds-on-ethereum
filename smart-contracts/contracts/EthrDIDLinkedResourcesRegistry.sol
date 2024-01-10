// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./EthereumDIDRegistry.sol";

/// Contract for storing and retrieving immutable resources (e.g. anoncreds assets)
/// uploaded by an authenticated signer [address].
/// TODO
contract EthrDIDLinkedResourcesRegistry {

    EthereumDIDRegistry public didRegistry;

    /// storage of string blobs, which are immutable once uploaded, and identified by its name + DID Identity
    mapping(address => mapping(string => bytes)) private immutableResourceByNameByDidIdentity;

    mapping(address => mapping(string => MutableResource)) private mutableResourceByNameByDidIdentity;

    mapping(address => mapping(string => MutableResourceUpdateMetadata[])) private mutableResourceUpdateMetadataByNameByDidIdentity;

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
    event ImmutableResourceCreatedEvent(address didIdentity, string name);
    event MutableResourceUpdatedEvent(address indexed didIdentity, string indexed indexedName, string name, MutableResource resource);

    constructor(address didRegistryAddress) {
        didRegistry = EthereumDIDRegistry(didRegistryAddress);
    }

    function doesImmutableResourceExist(address didIdentity, string memory name) private view returns (bool) {
        bytes memory resource = immutableResourceByNameByDidIdentity[didIdentity][name];
        return resource.length != 0;
    }

    /// Store [content] as an immutable resource in this registry. Where [content] is uniquely identified
    /// by the [name] and the DID identity.
    /// Note that since this is immutable data, repeated [name]s can only be used once per given DID Identity.
    function createImmutableResource(address didIdentity, string memory name, bytes memory content) public onlyDidIdentityOwner(didIdentity) {
        require(!doesImmutableResourceExist(didIdentity, name), "Resource already created with this Name and DID");
        immutableResourceByNameByDidIdentity[didIdentity][name] = content;
        emit ImmutableResourceCreatedEvent(didIdentity, name);
    }

    /// Get the [content] of an immutable resource in this registry, identified by it's [name] and [didIdentity].
    function getImmutableResource(address didIdentity, string memory name) public view returns (bytes memory) {
        return immutableResourceByNameByDidIdentity[didIdentity][name];
    }

    /// Update the mutable resource at the given [name] and [didIdentity] with the given [content].
    function updateMutableResource(address didIdentity, string memory name, bytes memory content) public onlyDidIdentityOwner(didIdentity) {
        uint32 blockTimestamp = uint32(block.timestamp);
        uint32 blockNumber = uint32(block.number);

        MutableResource memory previousResource = mutableResourceByNameByDidIdentity[didIdentity][name];

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
        mutableResourceUpdateMetadataByNameByDidIdentity[didIdentity][name].push(metadata);

        // set the new resource
        mutableResourceByNameByDidIdentity[didIdentity][name] = resource;
        emit MutableResourceUpdatedEvent(didIdentity, name, name, resource);
    }

    function getCurrentMutableResource(address didIdentity, string memory name) public view returns (MutableResource memory) {
        return mutableResourceByNameByDidIdentity[didIdentity][name];
    }

    function getMutableResourceUpdatesMetadata(address didIdentity, string memory name) public view returns (MutableResourceUpdateMetadata[] memory) {
        return mutableResourceUpdateMetadataByNameByDidIdentity[didIdentity][name];
    }
}
