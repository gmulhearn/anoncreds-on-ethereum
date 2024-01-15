use chrono::offset::Utc;
use chrono::DateTime;

/// Resource struct represents a resource with various properties.
///
/// https://wiki.trustoverip.org/display/HOME/DID-Linked+Resources+Specification
#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    // TODO - content isn't really apart of the Resource properties. 
    // May need to differentiate between Resource and ResourceContent
    pub content: Vec<u8>,
    /// A string or a map that conforms to the rules of [RFC3986] for URIs which SHOULD directly lead to a location where the resource can be accessed from.
    /// For example: did:example:46e2af9a-2ea0-4815-999d-730a6778227c/resources/0f964a80-5d18-4867-83e3-b47f5a756f02, or, https://gateway.ipfs.io/ipfs/bafybeihetj2ng3d74k7t754atv2s5dk76pcqtvxls6dntef3xa6rax25xe
    pub resource_uri: String,
    /// A string that identifies the type of resource. This property, along with the resourceName above, can be used to track version changes within a resource. Not to be confused with media type. (TBC to add to DID Spec Registries)
    /// For example: JSONSchema2020
    pub resource_type: String,
    /// A string that uniquely names and identifies a resource. This property, along with the resourceType below, can be used to track version changes within a resource.
    /// For example: degreeLaw
    pub resource_name: String,
    /// A string that conforms to a method specific unique identifier format.
    /// For example: 0f964a80-5d18-4867-83e3-b47f5a756f02
    pub resource_id: Option<String>,
    /// A string that conforms to a method specific unique identifier format.
    /// For example: 46e2af9a-2ea0-4815-999d-730a6778227c
    pub resource_collection_id: Option<String>,
    /// A string that uniquely identifies the version of the resource provided by the resource creator as a tag.
    /// For example: 1.3.1
    pub resource_version_id: Option<String>,
    /// A string that identifies the IANA-registered Media Type for a resource.
    /// For example: application/json
    pub media_type: String,
    /// A JSON String serialized as an XML Datetime normalized to UTC 00:00:00 and without sub-second decimal precision.
    /// For example: 2020-12-20T19:17:47Z
    pub created: DateTime<Utc>,
    /// A string that provides a checksum (e.g. SHA256, MD5) for the resource to facilitate data integrity.
    /// For example: 7b2022636f6e74656e74223a202274657374206461746122207d0ae3b0c44298
    pub checksum: Option<String>,
    /// The value of the property MUST be an string. This is the previous version of a resource with the same resourceName and resourceType. The value must be 'null' if there is no previous version.
    /// For example: 67618cfa-7a1d-4be3-b9b2-3a9ea52af305
    pub previous_version_id: Option<String>,
    /// The value of the property MUST be an string. The value must be 'null' if there is no next version.
    /// For example: null
    pub next_version_id: Option<String>,
}
