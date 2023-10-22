// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract AnoncredsRegistry {

    // storage of string blobs, which are immutable once uploaded, and identified by it's ID + Author
    mapping(address => mapping(string => string)) immutableResourceByIdByAuthorAddress;

    // note that by issuer (address =>) is only needed for security purposes
    // where revStatusUpdateTimestamps[i] == the timestamp of revStatusLists[i] below
    mapping(address => mapping(string => uint[])) revStatusUpdateTimestampsByRevRegIdByIssuer;

    // note that by issuer (address =>) is only needed for security purposes
    mapping(address => mapping(string => RevocationStatusList[])) revStatusListsByRevRegIdByIssuer;
    
    // simplified revocation status list. Containing only the data we care about
    struct RevocationStatusList {
        string revocationList; // serialized bitvec (RON notation i believe)
        string currentAccumulator;
    }

    event NewResource(address issuer, string id);
    event NewRevRegStatusUpdate(string rev_reg_id, uint index_in_status_list, uint timestamp);

    constructor() {
    }

    function does_immutable_resource_exist(address author, string memory id) private view returns (bool) {
        string memory resource = immutableResourceByIdByAuthorAddress[author][id];
        return bytes(resource).length != 0;
    }

    function create_immutable_resource(string memory id, string memory content) public {
        address author = msg.sender;

        require(!does_immutable_resource_exist(author, id), "Resource already created with this ID and author");
        immutableResourceByIdByAuthorAddress[author][id] = content;
        emit NewResource(author, id);
    }

    function get_immutable_resource(address author, string memory id) public view returns (string memory) {
        return immutableResourceByIdByAuthorAddress[author][id];
    }

    function add_rev_reg_status_update(string memory rev_reg_id, RevocationStatusList memory status_list) public {
        address issuer = msg.sender;
        uint timestamp = block.timestamp;

        revStatusUpdateTimestampsByRevRegIdByIssuer[issuer][rev_reg_id].push(timestamp);
        revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id].push(status_list);

        uint newListLength = revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id].length;
        uint indexOfNewEntry = newListLength - 1;

        emit NewRevRegStatusUpdate(rev_reg_id, indexOfNewEntry, timestamp);
    }

    function get_rev_reg_update_timestamps(address issuer, string memory rev_reg_id) public view returns (uint[] memory) {
        return revStatusUpdateTimestampsByRevRegIdByIssuer[issuer][rev_reg_id];
    }

    function get_rev_reg_update_at_index(address issuer, string memory rev_reg_id, uint index) public view returns (RevocationStatusList memory) {
        return revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id][index];
    }
}
