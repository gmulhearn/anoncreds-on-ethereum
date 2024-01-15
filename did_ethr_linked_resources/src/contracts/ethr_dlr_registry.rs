use std::sync::Arc;

use ethers::abi::RawLog;
use ethers::contract::EthEvent;
use ethers::types::H160;
use ethers::{abi::Address, providers::Middleware, types::U256};

use crate::types::input::ResourceInput;
use crate::utils::full_did_into_did_identity;

use super::get_read_only_ethers_client;

// Include generated contract types from build script
include!(concat!(env!("OUT_DIR"), "/ethr_dlr_registry_contract.rs"));

// Address of the `EthrDLRRegistry.sol` smart contract to use
// (should copy and paste the address value after a hardhat deploy script)
pub const ETHR_DLR_REGISTRY_ADDRESS: &str = "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0";

pub fn contract_with_client<T: Middleware>(client: Arc<T>) -> EthrDLRRegistry<T> {
    EthrDLRRegistry::new(
        ETHR_DLR_REGISTRY_ADDRESS.parse::<Address>().unwrap(),
        client,
    )
}

pub struct DLRRegistry;

impl DLRRegistry {
    pub async fn create_or_update_resource(
        &self,
        signer: Arc<impl Middleware>,
        did: &str,
        resource: ResourceInput,
    ) -> NewResourceFilter {
        let contract = contract_with_client(signer);

        let did_identity = full_did_into_did_identity(did);

        let tx = contract
            .create_resource(
                did_identity,
                resource.resource_name,
                resource.resource_type,
                resource.resource_version_id,
                resource.media_type,
                resource.content.into(),
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        let resource_update_event = tx
            .logs
            .into_iter()
            .find_map(|log| {
                let event = NewResourceFilter::decode_log(&RawLog::from(log));
                event.ok()
            })
            .unwrap();

        resource_update_event
    }

    pub async fn get_resource_by_id(
        &self,
        did: &str,
        resource_id: &str,
    ) -> Option<NewResourceFilter> {
        let did_identity = full_did_into_did_identity(did);
        let resource_id = U256::from_dec_str(resource_id).unwrap();

        self.get_resource_by_id_raw(did_identity, resource_id).await
    }

    async fn get_resource_by_id_raw(
        &self,
        did_identity: H160,
        resource_id: U256,
    ) -> Option<NewResourceFilter> {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client);

        let mut precise_filter = contract.new_resource_filter();
        precise_filter.filter = precise_filter
            .filter
            .topic1(did_identity)
            .topic2(resource_id)
            .from_block(0);

        // Query this event filter on the contract
        let events: Vec<NewResourceFilter> = precise_filter.query().await.unwrap();
        let mut events = events.into_iter();

        return match (events.next(), events.next()) {
            (Some(event), None) => Some(event),
            (None, None) => None,
            _ => panic!("Multiple events found for resource id: {}", resource_id),
        };
    }

    pub async fn get_resource_by_name_and_type_at_epoch(
        &self,
        did: &str,
        resource_name: &str,
        resource_type: &str,
        epoch: u64,
    ) -> Option<(NewResourceFilter, ResourceVersionMetadataChainNode)> {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client.clone());

        let did_identity = full_did_into_did_identity(did);

        let resource_name_and_type = format!("{}{}", resource_name, resource_type);

        let metadata_chain: Vec<ResourceVersionMetadataChainNode> = contract
            .get_resource_metadata_chain(did_identity, resource_name_and_type)
            .call()
            .await
            .unwrap();

        let search_res =
            metadata_chain.binary_search_by(|node| node.created.block_timestamp.cmp(&epoch));

        let metadata_node = match search_res {
            Ok(idx) => metadata_chain.into_iter().nth(idx).unwrap(),
            Err(idx) => {
                if idx == 0 {
                    // this indicates that the epoch is before the first version
                    return None;
                }
                metadata_chain.into_iter().nth(idx - 1).unwrap()
            }
        };

        Some((
            self.get_resource_by_id_raw(did_identity, metadata_node.resource_id)
                .await
                .unwrap(),
            metadata_node,
        ))
    }

    pub async fn get_resource_metadata_chain_node(
        &self,
        did: &str,
        resource_name: &str,
        resource_type: &str,
        index: u64,
    ) -> ResourceVersionMetadataChainNode {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client.clone());

        let did_identity = full_did_into_did_identity(did);

        let resource_name_and_type = format!("{}{}", resource_name, resource_type);

        contract
            .get_resource_metadata_chain_node(
                did_identity,
                resource_name_and_type,
                U256::from(index),
            )
            .call()
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        contracts::test_utils::get_writer_ethers_client, types::input::ResourceInput,
        utils::did_identity_as_full_did,
    };

    use super::DLRRegistry;

    #[tokio::test]
    async fn testtest() {
        let signer = get_writer_ethers_client(0);
        let did = did_identity_as_full_did(&signer.address());

        let registry = DLRRegistry;

        let resource_name = &format!("foo{}", uuid::Uuid::new_v4());
        let resource_type = "bar";

        let resource1 = registry
            .create_or_update_resource(
                signer.clone(),
                &did,
                ResourceInput {
                    resource_name: resource_name.to_owned(),
                    resource_type: resource_type.to_owned(),
                    resource_version_id: String::new(),
                    media_type: String::from("text/plain"),
                    content: "hello world".as_bytes().to_vec(),
                },
            )
            .await;

        dbg!(&resource1);

        let resource2 = registry
            .create_or_update_resource(
                signer.clone(),
                &did,
                ResourceInput {
                    resource_name: resource_name.to_owned(),
                    resource_type: resource_type.to_owned(),
                    resource_version_id: String::new(),
                    media_type: String::from("text/plain"),
                    content: "hello world2".as_bytes().to_vec(),
                },
            )
            .await;

        dbg!(&resource2);

        let resource3 = registry
            .create_or_update_resource(
                signer.clone(),
                &did,
                ResourceInput {
                    resource_name: resource_name.to_owned(),
                    resource_type: resource_type.to_owned(),
                    resource_version_id: String::new(),
                    media_type: String::from("text/plain"),
                    content: "hello world2".as_bytes().to_vec(),
                },
            )
            .await;

        dbg!(&resource3);

        let fetched_res1 = registry
            .get_resource_by_id(&did, &resource1.resource.resource_id.to_string())
            .await
            .unwrap();

        dbg!(fetched_res1);

        let fetched_res2 = registry
            .get_resource_by_id(&did, &resource2.resource.resource_id.to_string())
            .await
            .unwrap();

        dbg!(fetched_res2);

        let fetched_res3 = registry
            .get_resource_by_id(&did, &resource3.resource.resource_id.to_string())
            .await
            .unwrap();

        dbg!(fetched_res3);

        let resource_1_timestamp = resource1.resource.metadata.created.block_timestamp;
        // let resource_2_timestamp = resource2.resource.metadata.created.block_timestamp;
        let resource_3_timestamp = resource3.resource.metadata.created.block_timestamp;

        for epoch_to_try in resource_1_timestamp - 2..resource_3_timestamp + 2 {
            dbg!(
                epoch_to_try,
                registry
                    .get_resource_by_name_and_type_at_epoch(
                        &did,
                        resource_name,
                        resource_type,
                        epoch_to_try
                    )
                    .await,
            );
        }
    }
}
