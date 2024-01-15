use chrono::{TimeZone, Utc};
use ethers::types::H160;

use crate::{
    contracts::ethr_dlr_registry::{NewResourceFilter, ResourceVersionMetadataChainNode},
    types::output::Resource,
};

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

pub fn extract_did_of_dlr_resource_uri(resource_uri: &str) -> String {
    resource_uri.split("/resources").next().unwrap().to_owned()
}

impl From<(NewResourceFilter, ResourceVersionMetadataChainNode)> for Resource {
    fn from((event, metadata_node): (NewResourceFilter, ResourceVersionMetadataChainNode)) -> Self {
        let ledger_resource = event.resource;
        let ledger_res_meta = ledger_resource.metadata;

        let did_identity = event.did_identity;

        let resource_uri = format!(
            "did:local:ethr:{did_identity:?}/resources/{resource_id}",
            resource_id = ledger_resource.resource_id
        );

        let created_epoch = ledger_res_meta.created.block_timestamp;

        let previous_version_id = match metadata_node.previous_resource_id.to_string().as_str() {
            "0" => None,
            x => Some(x.to_owned()),
        };

        let next_version_id = match metadata_node.next_resource_id.to_string().as_str() {
            "0" => None,
            x => Some(x.to_owned()),
        };

        let content = ledger_resource.content.to_vec();

        Resource {
            content,
            resource_uri,
            resource_type: ledger_res_meta.resource_type,
            resource_name: ledger_res_meta.resource_name,
            resource_id: Some(ledger_resource.resource_id.to_string()),
            resource_collection_id: Some(format!("{did_identity:?}")),
            resource_version_id: Some(ledger_res_meta.resource_version),
            media_type: ledger_res_meta.media_type,
            created: Utc.timestamp_opt(created_epoch as i64, 0).unwrap(),
            checksum: None,
            previous_version_id,
            next_version_id,
        }
    }
}
