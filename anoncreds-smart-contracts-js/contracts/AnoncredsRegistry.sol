// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract AnoncredsRegistry {

    // storage of string blobs, which are immutable once uploaded, and identified by it's ID + Author
    mapping(address => mapping(string => string)) immutableResourceByIdByAuthorAddress;

    mapping(address => mapping(string => mapping(string => string))) schemaJsonByVersionByNameByIssuerAddress;
    mapping(address => mapping(string => mapping(string => string))) credDefJsonByTagBySchemaIdByIssuerAddress;
    mapping(address => mapping(string => mapping(string => string))) revRegDefJsonByTagByCredDefIdByIssuerAddress;

    // simplified revocation status list. Containing only the data we care about
    struct RevocationStatusList {
        string revocationList; // serialized bitvec (RON notation i believe)
        string currentAccumulator;
    }

    // note that by issuer (address =>) is only needed for security purposes
    // where revStatusUpdateTimestamps[i] == the timestamp of revStatusLists[i] below
    mapping(address => mapping(string => uint[])) revStatusUpdateTimestampsByRevRegIdByIssuer;

    // note that by issuer (address =>) is only needed for security purposes
    mapping(address => mapping(string => RevocationStatusList[])) revStatusListsByRevRegIdByIssuer;

    event NewResource(address issuer, string id);
    event NewSchema(address issuer, string name, string version);
    event NewCredDef(address issuer, string schema_id, string tag);
    event NewRevRegDef(address issuer, string cred_def_id, string tag);
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

    function does_schema_exist(address issuer, string memory name, string memory version) private view returns (bool) {
        string memory entry = schemaJsonByVersionByNameByIssuerAddress[issuer][name][version];
        return bytes(entry).length != 0;
    }

    function create_schema(string memory name, string memory version, string memory schema_json) public {
        address issuer = msg.sender;

        require(!does_schema_exist(issuer, name, version), "Schema already created with this name, version and issuer");
        schemaJsonByVersionByNameByIssuerAddress[issuer][name][version] = schema_json;
        emit NewSchema(issuer, name, version);
    }

    function get_schema(address issuer, string memory name, string memory version) public view returns (string memory) {
        return schemaJsonByVersionByNameByIssuerAddress[issuer][name][version];
    }


    function does_cred_def_exist(address issuer, string memory schema_id, string memory tag) private view returns (bool) {
        string memory entry = credDefJsonByTagBySchemaIdByIssuerAddress[issuer][schema_id][tag];
        return bytes(entry).length != 0;
    }

    function create_cred_def(string memory schema_id, string memory tag, string memory cred_def_json) public {
        address issuer = msg.sender;

        require(!does_cred_def_exist(issuer, schema_id, tag), "Cre Def already created with this tag, schema and issuer");
        credDefJsonByTagBySchemaIdByIssuerAddress[issuer][schema_id][tag] = cred_def_json;
        emit NewCredDef(issuer, schema_id, tag);
    }

    function get_cred_def(address issuer, string memory schema_id, string memory tag) public view returns (string memory) {
        return credDefJsonByTagBySchemaIdByIssuerAddress[issuer][schema_id][tag];
    }


    function does_rev_reg_def_exist(address issuer, string memory cred_def_id, string memory tag) private view returns (bool) {
        string memory entry = revRegDefJsonByTagByCredDefIdByIssuerAddress[issuer][cred_def_id][tag];
        return bytes(entry).length != 0;
    }

    function create_rev_reg_def(string memory cred_def_id, string memory tag, string memory rev_reg_def_json) public {
        address issuer = msg.sender;

        require(!does_rev_reg_def_exist(issuer, cred_def_id, tag), "Rev Reg Def already created with this tag, cred_def and issuer");
        revRegDefJsonByTagByCredDefIdByIssuerAddress[issuer][cred_def_id][tag] = rev_reg_def_json;
        emit NewRevRegDef(issuer, cred_def_id, tag);
    }

    function get_rev_reg_def(address issuer, string memory cred_def_id, string memory tag) public view returns (string memory) {
        return revRegDefJsonByTagByCredDefIdByIssuerAddress[issuer][cred_def_id][tag];
    }

    function add_rev_reg_status_update(string memory rev_reg_id, RevocationStatusList memory status_list) public {
        address issuer = msg.sender;
        uint timestamp = block.timestamp;

        revStatusUpdateTimestampsByRevRegIdByIssuer[issuer][rev_reg_id].push(timestamp);
        revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id].push(status_list);
    }

    function get_rev_reg_update_timestamps(address issuer, string memory rev_reg_id) public view returns (uint[] memory) {
        return revStatusUpdateTimestampsByRevRegIdByIssuer[issuer][rev_reg_id];
    }

    function get_rev_reg_update_at_index(address issuer, string memory rev_reg_id, uint index) public view returns (RevocationStatusList memory) {
        return revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id][index];
    }
}
