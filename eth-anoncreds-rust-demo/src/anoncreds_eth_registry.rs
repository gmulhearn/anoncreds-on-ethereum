use std::env;
use std::error::Error;
use std::sync::Arc;

use anyhow::anyhow;
use dotenv::dotenv;
use ethers::contract::EthLogDecode;
use ethers::providers::Middleware;
use ethers::signers::coins_bip39::English;
use ethers::signers::{MnemonicBuilder, Signer};
use ethers::types::{Address, U256};
use ethers::utils::keccak256;
use ethers::{
    abi::RawLog,
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::H160,
};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::ledger::ledger_data::LedgerData;
use crate::ledger::status_list_update_ledger_data::StatusListUpdateLedgerData;
#[cfg(feature = "thegraph")]
use crate::subgraph_query;

// Include generated contract types from build script
include!(concat!(env!("OUT_DIR"), "/anoncreds_registry_contract.rs"));

// Ethereum RPC of the network to use (defaults to the hardhat local network)
pub const REGISTRY_RPC: &str = "http://localhost:8545";

// Address of the `AnoncredsRegistry` smart contract to use
// (should copy and paste the address value after a hardhat deploy script)
pub const DEFAULT_ANONCRED_REGISTRY_ADDRESS: &str = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";

pub const ETHR_DID_SUB_METHOD: &str = "gmtest";

pub type EtherSigner = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

pub fn get_writer_ethers_client(id: u32) -> Arc<EtherSigner> {
    dotenv().ok();

    let seed = env::var("MNEMONIC").unwrap();

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(&*seed)
        .index(id)
        .unwrap()
        .build()
        .unwrap()
        .with_chain_id(31337 as u64);

    let provider = Provider::<Http>::try_from(REGISTRY_RPC).unwrap();
    Arc::new(SignerMiddleware::new(provider, wallet))
}

pub fn get_read_only_ethers_client() -> Arc<impl Middleware> {
    let provider = Provider::<Http>::try_from(REGISTRY_RPC).unwrap();
    Arc::new(provider)
}

pub fn contract_with_client<T: Middleware>(client: Arc<T>) -> AnoncredsRegistry<T> {
    let anoncreds_contract_address = env::var("ANONCRED_REGISTRY_ADDRESS")
        .unwrap_or(DEFAULT_ANONCRED_REGISTRY_ADDRESS.to_owned());
    AnoncredsRegistry::new(
        anoncreds_contract_address.parse::<Address>().unwrap(),
        client,
    )
}

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

/// Represents an identifier for an immutable resource stored in the registry.
#[derive(Debug)]
pub struct DIDResourceId {
    pub did_identity: H160,
    pub resource_path: String,
}

impl DIDResourceId {
    pub fn from_id(id: String) -> Self {
        let Some((did, resource_path)) = id.split_once("/") else {
            panic!("Could not process as DID Resource: {id}")
        };

        let did_identity_hex_str = did
            .split(":")
            .last()
            .expect(&format!("Could not read find author of DID: {did}"));
        let did_identity = did_identity_hex_str.parse().unwrap();

        DIDResourceId {
            did_identity,
            resource_path: resource_path.to_owned(),
        }
    }

    pub fn to_id(&self) -> String {
        let did = self.author_did();
        format!("{}/{}", did, self.resource_path)
    }

    pub fn author_did(&self) -> String {
        did_identity_as_full_did(&self.did_identity)
    }
}

pub struct AnoncredsEthRegistry;

impl AnoncredsEthRegistry {
    pub fn new() -> Self {
        AnoncredsEthRegistry {}
    }

    /// Push any JSON serializable [resource] to the registry as an immutable resource.
    ///
    /// The immutable resource is published using the signer passed in. For this transaction
    /// to succeed, the signer should be the controller of the given `did`.
    ///
    /// The resource is given a random ID, under the provided [parent_path].
    ///
    /// Returns the resource identifier for the pushed resource.
    pub async fn submit_immutable_json_resource<S>(
        &self,
        signer: Arc<impl Middleware>,
        did: &str,
        resource: &S,
        parent_path: &str,
    ) -> Result<DIDResourceId, Box<dyn Error>>
    where
        S: Serialize,
    {
        let contract = contract_with_client(signer);

        let resource_json = serde_json::to_string(resource).unwrap();
        let resource_path = format!("{parent_path}/{}", Uuid::new_v4());
        let did_identity = full_did_into_did_identity(did);

        contract
            .create_immutable_resource(did_identity.clone(), resource_path.clone(), resource_json)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .await
            .map_err(|e| anyhow!("{e:?}"))?;

        Ok(DIDResourceId {
            did_identity,
            resource_path,
        })
    }

    /// Find an immutable resource within the registry, as identified by [resource_id],
    /// then JSON deserialize the resource contents into type [D].
    pub async fn get_immutable_json_resource<D>(&self, resource_id: &str) -> D
    where
        D: DeserializeOwned,
    {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client);

        let did_resource_parts = DIDResourceId::from_id(resource_id.to_owned());

        let resource_json: String = contract
            .get_immutable_resource(
                did_resource_parts.did_identity,
                did_resource_parts.resource_path,
            )
            .call()
            .await
            .unwrap();

        serde_json::from_str(&resource_json).unwrap()
    }

    pub async fn submit_mutable_resource<T>(
        &self,
        signer: Arc<impl Middleware>,
        did: &str,
        resource: T,
        resource_path: &str,
    ) -> Result<u64, Box<dyn Error>>
    where
        T: LedgerData,
    {
        let contract = contract_with_client(signer);

        let resource_bytes = resource.into_ledger_bytes();
        let did_identity = full_did_into_did_identity(did);

        let tx = contract
            .update_mutable_resource(
                did_identity.clone(),
                resource_path.to_owned(),
                ethers::types::Bytes::from(resource_bytes),
            )
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .unwrap();

        // extract the emitted [AnoncredsRegistryEvents::StatusListUpdateEventFilter] event
        // from the transaction's receipt. This event importantly contains the ledger
        // timestamp that will be used for that revocation status list entry.
        let status_list_update_event = tx
            .logs
            .into_iter()
            .find_map(|log| {
                let contract_event = AnoncredsRegistryEvents::decode_log(&RawLog::from(log));
                match contract_event {
                    Ok(AnoncredsRegistryEvents::MutableResourceUpdateEventFilter(inner)) => {
                        Some(inner)
                    }
                    _ => None,
                }
            })
            .unwrap();

        let ledger_recorded_timestamp = status_list_update_event.resource.metadata.block_timestamp;

        Ok(ledger_recorded_timestamp)
    }

    /// This function works by doing the following:
    /// 1. get ALL resource update metadatas from the ledger
    /// 2. from those metadatas, find the timestamp & block number closest to [timestamp]
    /// 3. query the ledger for a resource update event for the resource_id and block number from 2.
    /// 4. reconstruct the data from the found event
    #[allow(unreachable_code)]
    pub async fn get_mutable_resource_as_of_timestamp<T>(
        &self,
        resource_id: &str,
        timestamp: u64,
    ) -> (T, u64)
    where
        T: LedgerData,
    {
        #[cfg(feature = "thegraph")]
        return self
            .get_mutable_resource_as_of_timestamp_via_subgraph::<T>(resource_id, timestamp)
            .await;

        self.get_mutable_resource_as_of_timestamp_via_pure_ethereum_api::<T>(resource_id, timestamp)
            .await
    }

    async fn get_mutable_resource_as_of_timestamp_via_subgraph<T>(
        &self,
        resource_id: &str,
        timestamp: u64,
    ) -> (T, u64)
    where
        T: LedgerData,
    {
        let resource_id = DIDResourceId::from_id(resource_id.to_owned());

        let res =
            subgraph_query::get_resource_update_event_most_recent_to(resource_id, timestamp).await;

        let resource_bytes = hex_to_bytes(&res.content_hex);

        (
            T::from_ledger_bytes(&resource_bytes),
            res.timestamp.parse().unwrap(),
        )
    }

    async fn get_mutable_resource_as_of_timestamp_via_pure_ethereum_api<T>(
        &self,
        resource_id: &str,
        timestamp: u64,
    ) -> (T, u64)
    where
        T: LedgerData,
    {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client.clone());

        let did_resource_parts = DIDResourceId::from_id(resource_id.to_owned());
        let resource_path = did_resource_parts.resource_path.clone();

        // get the metadata (timestamp + block number) for all mutable resource updates that have been made.
        let all_updates_metadata: Vec<MutableResourceUpdateMetadata> = contract
            .get_mutable_resource_updates_metadata(
                did_resource_parts.did_identity,
                resource_path.clone(),
            )
            .call()
            .await
            .unwrap();

        if all_updates_metadata.is_empty() {
            panic!("No update entries for resource: {resource_path}")
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
        // for the specific resource path + did identity and for the exact block number. This should result
        // in the exact resource update we want being found.
        let precise_status_list_update_event_filter = contract
            .mutable_resource_update_event_filter()
            .filter
            .topic1(did_resource_parts.did_identity)
            .topic2(U256::from(keccak256(resource_path)))
            .from_block(suitable_update_block_number)
            .to_block(suitable_update_block_number);

        // Query this event filter on the contract
        let filtered_resource_update_events: Vec<_> = client
            .get_logs(&precise_status_list_update_event_filter)
            .await
            .unwrap()
            .into_iter()
            .filter_map(|log| {
                let contract_event = AnoncredsRegistryEvents::decode_log(&RawLog::from(log));
                match contract_event {
                    Ok(AnoncredsRegistryEvents::MutableResourceUpdateEventFilter(inner)) => {
                        Some(inner)
                    }
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

    /// For the given [rev_reg_id], find the revocation status list entry that is
    /// closest to the given [timestamp], but no later.
    ///
    /// Returns the closest revocation status list, and the timestamp of that revocation
    /// status list entry. (The timestamp is also within the status list object, but is
    /// not accessible).
    #[allow(unreachable_code)]
    pub async fn get_rev_reg_status_list_as_of_timestamp(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> (anoncreds::types::RevocationStatusList, u64) {
        let (data, actual_timestamp) = self
            .get_mutable_resource_as_of_timestamp::<StatusListUpdateLedgerData>(
                rev_reg_id, timestamp,
            )
            .await;

        let anoncreds_data = data.into_anoncreds_data(timestamp, rev_reg_id);

        (anoncreds_data, actual_timestamp)
    }
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex_without_prefix = hex.trim_start_matches("0x");
    hex::decode(hex_without_prefix).unwrap()
}
