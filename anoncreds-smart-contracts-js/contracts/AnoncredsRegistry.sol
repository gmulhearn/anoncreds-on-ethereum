// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract AnoncredsRegistry {

    mapping(address => mapping(string => mapping(string => string))) schema_json_by_version_by_name_by_issuer_address;
    mapping(address => mapping(string => mapping(string => string))) cred_def_json_by_tag_by_schema_id_by_issuer_address;

    event NewSchema(address issuer, string name, string version);
    event NewCredDef(address issuer, string schema_id, string tag);

    constructor() {
    }

    function does_schema_exist(address issuer, string memory name, string memory version) private view returns (bool) {
        string memory entry = schema_json_by_version_by_name_by_issuer_address[issuer][name][version];
        return bytes(entry).length != 0;
    }

    function create_schema(string memory name, string memory version, string memory schema_json) public {
        address issuer = msg.sender;

        require(!does_schema_exist(issuer, name, version), "Schema already created with this name version and issuer");
        schema_json_by_version_by_name_by_issuer_address[issuer][name][version] = schema_json;
        emit NewSchema(issuer, name, version);
    }

    function get_schema(address issuer, string memory name, string memory version) public view returns (string memory) {
        return schema_json_by_version_by_name_by_issuer_address[issuer][name][version];
    }

    function does_cred_def_exist(address issuer, string memory schema_id, string memory tag) private view returns (bool) {
        string memory entry = cred_def_json_by_tag_by_schema_id_by_issuer_address[issuer][schema_id][tag];
        return bytes(entry).length != 0;
    }

    function create_cred_def(string memory schema_id, string memory tag, string memory cred_def_json) public {
        address issuer = msg.sender;

        require(!does_cred_def_exist(issuer, schema_id, tag), "Cre Def already created with this tag schema and issuer");
        cred_def_json_by_tag_by_schema_id_by_issuer_address[issuer][schema_id][tag] = cred_def_json;
        emit NewCredDef(issuer, schema_id, tag);
    }

    function get_cred_def(address issuer, string memory schema_id, string memory tag) public view returns (string memory) {
        return cred_def_json_by_tag_by_schema_id_by_issuer_address[issuer][schema_id][tag];
    }
}
