use ethers::types::H160;

const ETHR_DID_SUB_METHOD: &str = "local";

pub fn did_identity_as_full_did(address: &H160) -> String {
    // note that debug fmt of address is the '0x..' hex encoding.
    // where as .to_string() (fmt) truncates it
    format!("did:ethr:{ETHR_DID_SUB_METHOD}:{address:?}")
}

pub fn full_did_into_did_identity(did: &str) -> H160 {
    let identity_hex_str = did
        .split(":")
        .last()
        .expect(&format!("Could not read find identity of DID: {did}"));
    identity_hex_str.parse().unwrap()
}

/// Represents an identifier for an resource (immutable or mutable) stored in the registry.
#[derive(Debug)]
pub struct DIDLinkedResourceId {
    pub did_identity: H160,
    pub resource_type: DIDLinkedResourceType,
    pub resource_name: String,
}

#[derive(Debug)]
pub enum DIDLinkedResourceType {
    Immutable,
    Mutable,
}

impl DIDLinkedResourceId {
    pub fn from_full_id(id: String) -> Self {
        let Some((did, resource_path)) = id.split_once("/") else {
            panic!("Could not process as DID Resource: {id}")
        };

        let (resource_type, resource_identifier) =
            if let Some(rest) = resource_path.strip_prefix("resource/immutable/") {
                (DIDLinkedResourceType::Immutable, rest.to_owned())
            } else if let Some(rest) = resource_path.strip_prefix("resource/mutable/") {
                (DIDLinkedResourceType::Mutable, rest.to_owned())
            } else {
                panic!("Could not process as DID Resource: {id}")
            };

        let did_identity_hex_str = did
            .split(":")
            .last()
            .expect(&format!("Could not read find author of DID: {did}"));
        let did_identity = did_identity_hex_str.parse().unwrap();

        DIDLinkedResourceId {
            did_identity,
            resource_type,
            resource_name: resource_identifier,
        }
    }

    pub fn to_full_id(&self) -> String {
        let did = self.author_did();

        let resource_type = match self.resource_type {
            DIDLinkedResourceType::Immutable => "immutable",
            DIDLinkedResourceType::Mutable => "mutable",
        };

        format!(
            "{}/resource/{}/{}",
            did, resource_type, self.resource_name
        )
    }

    pub fn author_did(&self) -> String {
        did_identity_as_full_did(&self.did_identity)
    }
}
