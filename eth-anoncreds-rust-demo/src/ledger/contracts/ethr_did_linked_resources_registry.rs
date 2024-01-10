use std::env;
use std::error::Error;
use std::sync::Arc;

use anyhow::anyhow;
use ethers::abi::RawLog;
use ethers::contract::EthLogDecode;
use ethers::providers::Middleware;
use ethers::types::{Address, U256};
use ethers::utils::keccak256;

use crate::ledger::did_linked_resource_id::{
    full_did_into_did_identity, DIDLinkedResourceId, DIDLinkedResourceType,
};
use crate::ledger::ledger_data::LedgerDataTransformer;
#[cfg(feature = "thegraph")]
use crate::ledger::subgraph_query;

use super::get_read_only_ethers_client;

// Include generated contract types from build script
include!(concat!(
    env!("OUT_DIR"),
    "/ethr_did_linked_resources_registry_contract.rs"
));

// Address of the `EthrDIDLinkedResourcesRegistry` smart contract to use
// (should copy and paste the address value after a hardhat deploy script)
pub const DEFAULT_LINKED_RESOURCES_REGISTRY_ADDRESS: &str =
    "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";

pub const ETHR_DID_SUB_METHOD: &str = "gmtest";

pub fn contract_with_client<T: Middleware>(client: Arc<T>) -> EthrDIDLinkedResourcesRegistry<T> {
    let resources_contract_address = env::var("RESOURCES_REGISTRY_ADDRESS")
        .unwrap_or(DEFAULT_LINKED_RESOURCES_REGISTRY_ADDRESS.to_owned());
    EthrDIDLinkedResourcesRegistry::new(
        resources_contract_address.parse::<Address>().unwrap(),
        client,
    )
}

pub struct LinkedResourcesRegistry;

impl LinkedResourcesRegistry {
    pub fn new() -> Self {
        LinkedResourcesRegistry {}
    }

    /// Push any JSON serializable [resource] to the registry as an immutable resource.
    ///
    /// The immutable resource is published using the signer passed in. For this transaction
    /// to succeed, the signer should be the controller of the given `did`.
    ///
    /// Returns the resource identifier for the pushed resource.
    pub async fn submit_immutable_json_resource<T>(
        &self,
        signer: Arc<impl Middleware>,
        did: &str,
        resource: T,
        resource_name: &str,
    ) -> Result<DIDLinkedResourceId, Box<dyn Error>>
    where
        T: LedgerDataTransformer,
    {
        let contract = contract_with_client(signer);

        let resource_bytes = resource.into_ledger_bytes();
        let did_identity = full_did_into_did_identity(did);

        contract
            .create_immutable_resource(
                did_identity.clone(),
                resource_name.to_owned(),
                ethers::types::Bytes::from(resource_bytes),
            )
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .await
            .map_err(|e| anyhow!("{e:?}"))?;

        Ok(DIDLinkedResourceId {
            did_identity,
            resource_type: DIDLinkedResourceType::Immutable,
            resource_name: resource_name.to_owned(),
        })
    }

    /// Find an immutable resource within the registry, as identified by [resource_id],
    /// then JSON deserialize the resource contents into type [D].
    pub async fn get_immutable_json_resource<T>(&self, resource_id: &str) -> T
    where
        T: LedgerDataTransformer,
    {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client);

        let did_resource_parts = DIDLinkedResourceId::from_full_id(resource_id.to_owned());

        let resource_bytes: Vec<u8> = contract
            .get_immutable_resource(
                did_resource_parts.did_identity,
                did_resource_parts.resource_name,
            )
            .call()
            .await
            .unwrap()
            .0
            .to_vec();

        T::from_ledger_bytes(&resource_bytes)
    }

    pub async fn submit_mutable_resource<T>(
        &self,
        signer: Arc<impl Middleware>,
        did: &str,
        resource: T,
        resource_name: &str,
    ) -> Result<u64, Box<dyn Error>>
    where
        T: LedgerDataTransformer,
    {
        let contract = contract_with_client(signer);

        let resource_bytes = resource.into_ledger_bytes();
        let did_identity = full_did_into_did_identity(did);

        let tx = contract
            .update_mutable_resource(
                did_identity.clone(),
                resource_name.to_owned(),
                ethers::types::Bytes::from(resource_bytes),
            )
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .unwrap();

        // extract the emitted [EthrDIDLinkedResourcesRegistryEvents::MutableResourceUpdateEventFilter] event
        // from the transaction's receipt. This event importantly contains the ledger
        // timestamp that will be used for that resource update entry.
        let resource_update_event = tx
            .logs
            .into_iter()
            .find_map(|log| {
                let contract_event =
                    EthrDIDLinkedResourcesRegistryEvents::decode_log(&RawLog::from(log));
                match contract_event {
                    Ok(EthrDIDLinkedResourcesRegistryEvents::MutableResourceUpdatedEventFilter(
                        inner,
                    )) => Some(inner),
                    _ => None,
                }
            })
            .unwrap();

        let ledger_recorded_timestamp = resource_update_event.resource.metadata.block_timestamp;

        Ok(ledger_recorded_timestamp)
    }

    /// For the given [resource_id], find the mutable resource update that is
    /// closest to the given [timestamp], but no later.
    ///
    /// Returns the closest resource update, and the timestamp of that update.
    #[allow(unreachable_code)]
    pub async fn get_mutable_resource_as_of_timestamp<T>(
        &self,
        resource_id: &str,
        timestamp: u64,
    ) -> (T, u64)
    where
        T: LedgerDataTransformer,
    {
        #[cfg(feature = "thegraph")]
        return self
            .get_mutable_resource_as_of_timestamp_via_subgraph::<T>(resource_id, timestamp)
            .await;

        self.get_mutable_resource_as_of_timestamp_via_pure_ethereum_api::<T>(resource_id, timestamp)
            .await
    }

    #[cfg(feature = "thegraph")]
    async fn get_mutable_resource_as_of_timestamp_via_subgraph<T>(
        &self,
        resource_id: &str,
        timestamp: u64,
    ) -> (T, u64)
    where
        T: LedgerDataTransformer,
    {
        let resource_id = DIDLinkedResourceId::from_full_id(resource_id.to_owned());

        let res =
            subgraph_query::get_resource_update_event_most_recent_to(resource_id, timestamp).await;

        let resource_bytes = hex_to_bytes(&res.content_hex);

        (
            T::from_ledger_bytes(&resource_bytes),
            res.timestamp.parse().unwrap(),
        )
    }

    /// This function works by doing the following:
    /// 1. get ALL resource update metadatas from the ledger
    /// 2. from those metadatas, find the timestamp & block number closest to [timestamp]
    /// 3. query the ledger for a resource update event for the resource_id and block number from 2.
    /// 4. reconstruct the data from the found event
    async fn get_mutable_resource_as_of_timestamp_via_pure_ethereum_api<T>(
        &self,
        resource_id: &str,
        timestamp: u64,
    ) -> (T, u64)
    where
        T: LedgerDataTransformer,
    {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client.clone());

        let did_resource_parts = DIDLinkedResourceId::from_full_id(resource_id.to_owned());
        let resource_name = did_resource_parts.resource_name.clone();

        // get the metadata (timestamp + block number) for all mutable resource updates that have been made.
        let all_updates_metadata: Vec<MutableResourceUpdateMetadata> = contract
            .get_mutable_resource_updates_metadata(
                did_resource_parts.did_identity,
                resource_name.clone(),
            )
            .call()
            .await
            .unwrap();

        if all_updates_metadata.is_empty() {
            panic!("No update entries for resource: {resource_name}")
        }

        // TODO - here we might binary search rather than iter all
        // Find the index of the timestamp that is closest to the provided [timestamp]:
        // * scan the list until a timestamp is greater than the desired [timestamp],
        //  then minus one from the index of that item.
        // * OR, if no entries are greater than, just pick the last/latest entry.
        let index_of_suitable_update_metadata = all_updates_metadata
            .iter()
            .position(|ts| ts.block_timestamp as u64 > timestamp)
            .unwrap_or(all_updates_metadata.len())
            - 1;
        let suitable_update_metadata = &all_updates_metadata[index_of_suitable_update_metadata];
        let suitable_update_block_number = suitable_update_metadata.block_number;

        // Create an event filter for resource updates, filtering for update events
        // for the specific resource name + did identity and for the exact block number. This should result
        // in the exact resource update we want being found.
        let precise_resource_update_event_filter = contract
            .mutable_resource_updated_event_filter()
            .filter
            .topic1(did_resource_parts.did_identity)
            .topic2(U256::from(keccak256(resource_name)))
            .from_block(suitable_update_block_number)
            .to_block(suitable_update_block_number);

        // Query this event filter on the contract
        let filtered_resource_update_events: Vec<_> = client
            .get_logs(&precise_resource_update_event_filter)
            .await
            .unwrap()
            .into_iter()
            .filter_map(|log| {
                let contract_event =
                    EthrDIDLinkedResourcesRegistryEvents::decode_log(&RawLog::from(log));
                match contract_event {
                    Ok(EthrDIDLinkedResourcesRegistryEvents::MutableResourceUpdatedEventFilter(
                        inner,
                    )) => Some(inner),
                    _ => None,
                }
            })
            .collect();

        // assertion for sake of demo, proving that the filter worked without ambiguity
        assert!(filtered_resource_update_events.len() == 1);
        let resource_update_event = filtered_resource_update_events.into_iter().next().unwrap();

        let resource_bytes = resource_update_event.resource.content.0.to_vec();
        let update_timestamp = resource_update_event.resource.metadata.block_timestamp;
        (T::from_ledger_bytes(&resource_bytes), update_timestamp)
    }
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex_without_prefix = hex.trim_start_matches("0x");
    hex::decode(hex_without_prefix).unwrap()
}
