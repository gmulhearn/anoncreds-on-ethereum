use chrono::{TimeZone, Utc};
use ethers::types::H160;

use crate::{
    config::DidEthrSubMethod,
    contracts::ethr_dlr_registry::{NewResourceFilter, ResourceVersionMetadataChainNode},
    types::output::{Resource, ResourceMetadata},
};

/// sub method should be the hex string chain ID of the network, or a known "name":
/// https://github.com/uport-project/ethr-did-registry#contract-deployments
pub fn did_identity_as_full_did(address: &H160, sub_method: &DidEthrSubMethod) -> String {
    // note that debug fmt of address is the '0x..' hex encoding.
    // where as .to_string() (fmt) truncates it
    format!(
        "did:ethr:{sub_method}:{address:?}",
        sub_method = sub_method.0
    )
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

impl
    From<(
        NewResourceFilter,
        ResourceVersionMetadataChainNode,
        &DidEthrSubMethod,
    )> for Resource
{
    fn from(
        (event, metadata_node, sub_method): (
            NewResourceFilter,
            ResourceVersionMetadataChainNode,
            &DidEthrSubMethod,
        ),
    ) -> Self {
        let ledger_resource = event.resource;
        let ledger_res_meta = ledger_resource.metadata;

        let did_identity = event.did_identity;
        let did = did_identity_as_full_did(&did_identity, sub_method);

        let resource_uri = format!(
            "{did}/resources/{resource_id}",
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
            metadata: ResourceMetadata {
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
            },
        }
    }
}

#[cfg(feature = "thegraph")]
pub mod thegraph {
    use std::str::FromStr;

    use chrono::{TimeZone, Utc};
    use ethers::types::{H160, U256};

    use crate::{
        config::DidEthrSubMethod,
        contracts::ethr_dlr_registry::ResourceVersionMetadataChainNode,
        subgraph::query::ResourceForNameAndTypeAtTimestampQueryResult,
        types::output::{Resource, ResourceMetadata},
    };

    use super::did_identity_as_full_did;

    impl
        From<(
            ResourceForNameAndTypeAtTimestampQueryResult,
            ResourceVersionMetadataChainNode,
            &DidEthrSubMethod,
        )> for Resource
    {
        fn from(
            (event, metadata_node, sub_method): (
                ResourceForNameAndTypeAtTimestampQueryResult,
                ResourceVersionMetadataChainNode,
                &DidEthrSubMethod,
            ),
        ) -> Self {
            let did =
                did_identity_as_full_did(&H160::from_str(&event.did_identity).unwrap(), sub_method);
            let resource_uri = format!(
                "{did}/resources/{resource_id}",
                resource_id = event.resource_id
            );

            let created_epoch = U256::from_dec_str(&event.block_timestamp).unwrap().as_u64();

            let previous_version_id = match metadata_node.previous_resource_id.to_string().as_str()
            {
                "0" => None,
                x => Some(x.to_owned()),
            };

            let next_version_id = match metadata_node.next_resource_id.to_string().as_str() {
                "0" => None,
                x => Some(x.to_owned()),
            };

            let content = hex_to_bytes(&event.content);

            Resource {
                content,
                metadata: ResourceMetadata {
                    resource_uri,
                    resource_type: event.resource_type,
                    resource_name: event.resource_name,
                    resource_id: Some(event.resource_id),
                    resource_collection_id: Some(event.did_identity),
                    resource_version_id: Some(event.resource_version),
                    media_type: event.resource_media_type,
                    created: Utc.timestamp_opt(created_epoch as i64, 0).unwrap(),
                    checksum: None,
                    previous_version_id,
                    next_version_id,
                },
            }
        }
    }

    fn hex_to_bytes(hex_str: &str) -> Vec<u8> {
        let hex_str = hex_str.trim_start_matches("0x");
        hex::decode(hex_str).unwrap()
    }
}
