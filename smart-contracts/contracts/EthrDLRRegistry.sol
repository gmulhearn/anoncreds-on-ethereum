// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./EthereumDIDRegistry.sol";

/// TODO
contract EthrDLRRegistry {
    uint256 private globalResourceCounter = 0;

    EthereumDIDRegistry public didRegistry;

    modifier onlyDidIdentityOwner(address identity) {
        address actor = msg.sender;
        require (actor == didRegistry.identityOwner(identity), "bad_actor");
        _;
    }

    event NewResource(address indexed didIdentity, uint256 indexed resourceId, string indexed resourceNameAndType, Resource resource);

    // (didIdentity -> resourceName+resourceType -> ResourceVersionMetadataChainNode[])
    mapping(address => mapping(string => ResourceVersionMetadataChainNode[])) private resourceMetadataChains;

    struct Resource {
        uint256 resourceId;
        ResourceMetadata metadata;
        bytes content;
    }

    struct ResourceMetadata {
        string resourceName;
        string resourceType;
        string resourceVersion;
        string mediaType;
        LedgerTime created;
        uint256 metadataChainNodeIndex;
    }

    struct ResourceVersionMetadataChainNode {
        uint256 resourceId;
        LedgerTime created;
        uint256 nextResourceId; // is this neccessary?
        uint256 previousResourceId; // is this neccessary?
    }

    struct LedgerTime {
        uint40 blockTimestamp;
        uint64 blockNumber;
    }

    constructor(address didRegistryAddress) {
        didRegistry = EthereumDIDRegistry(didRegistryAddress);
    }

    function createResource(address didIdentity, string memory resourceName, string memory resourceType, string memory resourceVersion, string memory mediaType, bytes memory content) public onlyDidIdentityOwner(didIdentity) {
        globalResourceCounter++;

        uint256 resourceId = globalResourceCounter;
        string memory resourceNameAndType = string(abi.encodePacked(resourceName, resourceType));

        uint256 metadataChainNodeIndex = updateMetadataChain(didIdentity, resourceNameAndType, resourceId);

        LedgerTime memory createdLedgerTime = LedgerTime({
            blockTimestamp: uint40(block.timestamp),
            blockNumber: uint64(block.number)
        });

        Resource memory resource = Resource({
            resourceId: resourceId,
            metadata: ResourceMetadata({
                resourceName: resourceName,
                resourceType: resourceType,
                resourceVersion: resourceVersion,
                mediaType: mediaType,
                created: createdLedgerTime,
                metadataChainNodeIndex: metadataChainNodeIndex
            }),
            content: content
        });
        emit NewResource(didIdentity, resourceId, resourceNameAndType, resource);
    }

    function updateMetadataChain(address didIdentity, string memory resourceNameAndType, uint256 resourceId) private returns (uint256) {
        // update the previous resource version metadata chain node (if it exists)
        uint256 previousResourceId = 0;
        uint256 resourceMetadataChainLength = resourceMetadataChains[didIdentity][resourceNameAndType].length;

        if (resourceMetadataChainLength > 0) {
            // set
            resourceMetadataChains[didIdentity][resourceNameAndType][resourceMetadataChainLength - 1].nextResourceId = resourceId;
            // remember
            previousResourceId = resourceMetadataChains[didIdentity][resourceNameAndType][resourceMetadataChainLength - 1].resourceId;
        }

        // TODO - assert new ledger time is greater than previous ledger time

        // create the new (current) resource version metadata chain node
        ResourceVersionMetadataChainNode memory newResourceVersionMetadataChainNode = ResourceVersionMetadataChainNode({
            resourceId: resourceId,
            created: LedgerTime({
                blockTimestamp: uint40(block.timestamp),
                blockNumber: uint64(block.number)
            }),
            nextResourceId: 0,
            previousResourceId: previousResourceId
        });
        resourceMetadataChains[didIdentity][resourceNameAndType].push(newResourceVersionMetadataChainNode);

        // index is length pre push
        return resourceMetadataChainLength;
    }

    function getResourceMetadataChain(address didIdentity, string memory resourceNameAndType) public view returns (ResourceVersionMetadataChainNode[] memory) {
        return resourceMetadataChains[didIdentity][resourceNameAndType];
    }

    function getResourceMetadataChainLength(address didIdentity, string memory resourceNameAndType) public view returns (uint256) {
        return resourceMetadataChains[didIdentity][resourceNameAndType].length;
    }

    function getResourceMetadataChainNode(address didIdentity, string memory resourceNameAndType, uint256 index) public view returns (ResourceVersionMetadataChainNode memory) {
        return resourceMetadataChains[didIdentity][resourceNameAndType][index];
    }

    function getResourceMetadataChainSlice(address didIdentity, string memory resourceNameAndType, uint256 start, uint256 end) public view returns (ResourceVersionMetadataChainNode[] memory) {
        ResourceVersionMetadataChainNode[] memory resourceMetadataChainSlice = new ResourceVersionMetadataChainNode[](end - start);

        for (uint256 i = start; i < end; i++) {
            resourceMetadataChainSlice[i - start] = resourceMetadataChains[didIdentity][resourceNameAndType][i];
        }

        return resourceMetadataChainSlice;
    }
}