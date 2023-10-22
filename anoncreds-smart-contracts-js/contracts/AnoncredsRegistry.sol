// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

/// Contract for storing and retrieving immutable resources (e.g. anoncreds assets)
/// uploaded by an authenticated signer [address].
/// Also allow for storing and retrieving revocation status list updates, and some 
/// mechanisms for efficient lookups of revocation status lists by [timestamp].
contract AnoncredsRegistry {

    /// storage of string blobs, which are immutable once uploaded, and identified by it's ID + Author
    mapping(address => mapping(string => string)) immutableResourceByIdByAuthorAddress;

    /// where revStatusUpdateTimestamps[i] == the timestamp of revStatusLists[i] below
    /// note that by issuer (address =>) is only needed for security purposes.
    mapping(address => mapping(string => uint32[])) revStatusUpdateTimestampsByRevRegIdByIssuer;

    /// storage of revocation status lists, by the revocation registry ID, by the issuer.
    /// note that by issuer (address =>) is only needed for security purposes
    mapping(address => mapping(string => RevocationStatusList[])) revStatusListsByRevRegIdByIssuer;
    
    /// simplified revocation status list. Containing only the data we care about, 
    /// the rest can be constructed by the client with other metadata.
    struct RevocationStatusList {
        string revocationList; // serialized bitvec (RON notation i believe)
        string currentAccumulator;
    }

    event NewResource(address issuer, string id);
    event NewRevRegStatusUpdate(string rev_reg_id, uint index_in_status_list, uint32 timestamp);

    constructor() {
    }

    function does_immutable_resource_exist(address author, string memory id) private view returns (bool) {
        string memory resource = immutableResourceByIdByAuthorAddress[author][id];
        return bytes(resource).length != 0;
    }

    /// Store [content] as an immutable resource in this registry. Where [content] is uniquely identified
    /// by the [id] and the author (i.e. address that executed the transaction).
    /// Note that since this is immutable data, repeated [id]s can only be used once per given author.
    function create_immutable_resource(string memory id, string memory content) public {
        address author = msg.sender;

        require(!does_immutable_resource_exist(author, id), "Resource already created with this ID and author");
        immutableResourceByIdByAuthorAddress[author][id] = content;
        emit NewResource(author, id);
    }

    /// Get the [content] of an immutable resource in this registry, identifier by it's [id] and [author].
    function get_immutable_resource(address author, string memory id) public view returns (string memory) {
        return immutableResourceByIdByAuthorAddress[author][id];
    }

    /// Stores a new [status_list] within the list of status lists stored for the given [rev_reg_id] (and [issuer]).
    ///
    /// Emits an event, [NewRevRegStatusUpdate], which contains the registry-determined timestamp for the status_list
    /// entry.
    function add_rev_reg_status_update(string memory rev_reg_id, RevocationStatusList memory status_list) public {
        address issuer = msg.sender;
        uint32 timestamp = uint32(block.timestamp);

        revStatusUpdateTimestampsByRevRegIdByIssuer[issuer][rev_reg_id].push(timestamp);
        revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id].push(status_list);

        uint newListLength = revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id].length;
        uint indexOfNewEntry = newListLength - 1;

        emit NewRevRegStatusUpdate(rev_reg_id, indexOfNewEntry, timestamp);
    }

    /// Return the list of timestamps of revocation status list update that have been made for the given
    /// [rev_reg_id] and [issuer].
    /// This list will naturally be chronologically sorted.
    /// The indexes in this list are 1-to-1 with the full status list list. For instance, index "5" in this list
    /// may contain a timestamp like "1697948227", this indicates that the status list at index "5" has a timestamp
    /// of "1697948227".
    /// 
    /// The intention is that the data size of this list will be smaller than the entire list of revocation
    /// status list entries. So a consumer looking for a revocation status list entry near a certain timestamp
    /// can retrieve this list of timestamps, then find the index of their desired timestamp, then look up that 
    /// index to get the full [RevocationStatusList] details via [get_rev_reg_update_at_index].
    /// 
    /// Consumers may additionally wish to cache this list to avoid unneccessary future look ups.
    function get_rev_reg_update_timestamps(address issuer, string memory rev_reg_id) public view returns (uint32[] memory) {
        return revStatusUpdateTimestampsByRevRegIdByIssuer[issuer][rev_reg_id];
    }

    /// Returns the full [RevocationStatusList] entry information of a particular revocation registry at a particular index.
    ///
    /// consumers are intended to use [get_rev_reg_update_timestamps] to know exactly what [index] they are looking for.
    function get_rev_reg_update_at_index(address issuer, string memory rev_reg_id, uint index) public view returns (RevocationStatusList memory) {
        return revStatusListsByRevRegIdByIssuer[issuer][rev_reg_id][index];
    }
}
