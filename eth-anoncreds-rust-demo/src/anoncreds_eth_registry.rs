use std::env;
use std::error::Error;
use std::sync::Arc;

use anoncreds::data_types::issuer_id::IssuerId;
use anoncreds::data_types::rev_status_list::serde_revocation_list;
use anyhow::anyhow;
use bitvec::vec::BitVec;
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
use serde_json::{json, Value};
use uuid::Uuid;

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
    pub async fn submit_json_resource<S>(
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
    pub async fn get_json_resource<D>(&self, resource_id: &str) -> D
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

    /// Publish a new revocation status list for the revocation registry
    /// identifier by [rev_reg_id] to the registry.
    ///
    /// Returns the real timestamp recorded on the ledger
    pub async fn submit_rev_reg_status_list_update(
        &self,
        signer: Arc<impl Middleware>,
        did: &str,
        rev_reg_id: &str,
        revocation_status_list: &anoncreds::types::RevocationStatusList,
    ) -> u64 {
        let contract = contract_with_client(signer);

        let did_identity = full_did_into_did_identity(did);

        let ledger_status_list =
            construct_ledger_update_status_list_input_from_anoncreds_data(revocation_status_list);

        let tx = contract
            .update_revocation_registry_status_list(
                did_identity,
                String::from(rev_reg_id),
                ledger_status_list,
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
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
                    Ok(AnoncredsRegistryEvents::StatusListUpdateEventFilter(inner)) => Some(inner),
                    _ => None,
                }
            })
            .unwrap();

        let ledger_recorded_timestamp = status_list_update_event
            .status_list
            .metadata
            .block_timestamp as u64;

        ledger_recorded_timestamp
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
        #[cfg(feature = "thegraph")]
        return self
            .get_rev_reg_status_list_as_of_timestamp_via_subgraph(rev_reg_id, timestamp)
            .await;

        self.get_rev_reg_status_list_as_of_timestamp_via_pure_ethereum_api(rev_reg_id, timestamp)
            .await
    }

    /// This function works by doing the following:
    /// 1. get ALL status list update metadatas from the ledger
    /// 2. from those metadatas, find the timestamp & block number closest to [timestamp]
    /// 3. query the ledger for a status list update event for the rev_reg_id and block number from 2.
    /// 4. reconstruct the anoncreds data from the found event
    async fn get_rev_reg_status_list_as_of_timestamp_via_pure_ethereum_api(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> (anoncreds::types::RevocationStatusList, u64) {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client.clone());
        let rev_reg_resource_id = DIDResourceId::from_id(rev_reg_id.to_owned());

        // get the metadata (timestamp + block number) for all revocation status list updates that have been made.
        let all_updates_metadata: Vec<RevocationStatusListUpdateMetadata> = contract
            .get_revocation_registry_status_list_updates_metadata(
                rev_reg_resource_id.did_identity,
                String::from(rev_reg_id),
            )
            .call()
            .await
            .unwrap();

        if all_updates_metadata.is_empty() {
            panic!("No rev entries for rev reg: {rev_reg_id}")
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
        let suitable_update_timestamp: u64 = suitable_update_metadata.block_timestamp.into();
        let suitable_update_block_number = suitable_update_metadata.block_number;

        // Create an event filter for status list updates, filtering for update events
        // for the specific rev_reg_id and for the exact block number. This should result
        // in the exact status list update we want being found.
        let precise_status_list_update_event_filter = contract
            .status_list_update_event_filter()
            .filter
            .topic1(U256::from(keccak256(rev_reg_id)))
            .from_block(suitable_update_block_number)
            .to_block(suitable_update_block_number);

        // Query this event filter on the contract
        let filtered_status_list_update_events: Vec<_> = client
            .get_logs(&precise_status_list_update_event_filter)
            .await
            .unwrap()
            .into_iter()
            .filter_map(|log| {
                let contract_event = AnoncredsRegistryEvents::decode_log(&RawLog::from(log));
                match contract_event {
                    Ok(AnoncredsRegistryEvents::StatusListUpdateEventFilter(inner)) => Some(inner),
                    _ => None,
                }
            })
            .collect();

        // assertion for sake of demo, proving that the filter worked without ambiguity
        assert!(filtered_status_list_update_events.len() == 1);
        let status_list_update_event = filtered_status_list_update_events
            .into_iter()
            .next()
            .unwrap();

        // reconstruct the anoncreds RevocationStatusList from the ledger event data
        let rev_list = construct_anoncreds_status_list_from_ledger_event_data(
            rev_reg_id,
            &rev_reg_resource_id.author_did(),
            status_list_update_event
                .status_list
                .revocation_list_bit_vec
                .0
                .to_vec(),
            &status_list_update_event.status_list.current_accumulator,
            status_list_update_event
                .status_list
                .metadata
                .block_timestamp,
        );

        (rev_list, suitable_update_timestamp)
    }

    #[cfg(feature = "thegraph")]
    async fn get_rev_reg_status_list_as_of_timestamp_via_subgraph(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> (anoncreds::types::RevocationStatusList, u64) {
        let rev_reg_resource_id = DIDResourceId::from_id(rev_reg_id.to_owned());

        let res = subgraph_query::get_status_list_event_most_recent_to(rev_reg_id, timestamp).await;

        let timestamp: u32 = res.timestamp.parse().unwrap();

        let rev_list = construct_anoncreds_status_list_from_ledger_event_data(
            rev_reg_id,
            &rev_reg_resource_id.author_did(),
            hex_to_bytes(&res.status_list_hex),
            &res.current_accum,
            timestamp,
        );

        (rev_list, timestamp.into())
    }
}

// anoncreds type -> Ledger data type
fn construct_ledger_update_status_list_input_from_anoncreds_data(
    anoncreds_data: &anoncreds::types::RevocationStatusList,
) -> anoncreds_registry::UpdateRevocationStatusListInput {
    // dismantle the inner parts that we want to upload to the registry
    let revocation_status_list_json: Value = serde_json::to_value(anoncreds_data).unwrap();
    let current_accumulator = revocation_status_list_json
        .get("currentAccumulator")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    let revocation_list_val = revocation_status_list_json.get("revocationList").unwrap();
    let bitvec = serde_revocation_list::deserialize(revocation_list_val).unwrap();
    let bitvec_as_bytes = bitvec_to_bytes(bitvec);

    anoncreds_registry::UpdateRevocationStatusListInput {
        revocation_list_bit_vec: ethers::types::Bytes::from(bitvec_as_bytes),
        current_accumulator,
    }
}

// ledger event data type (plus other data) -> anoncreds type
fn construct_anoncreds_status_list_from_ledger_event_data(
    rev_reg_id: &str,
    did: &str,
    ledger_event_status_list_bit_vec: Vec<u8>,
    ledger_event_current_accum: &str,
    ledger_event_timestamp: u32,
) -> anoncreds::types::RevocationStatusList {
    let rev_list = bytes_to_bitvec(ledger_event_status_list_bit_vec);
    let current_accumulator = serde_json::from_value(json!(&ledger_event_current_accum)).unwrap();

    anoncreds::types::RevocationStatusList::new(
        Some(rev_reg_id),
        IssuerId::try_from(did).unwrap(),
        rev_list,
        Some(current_accumulator),
        Some(ledger_event_timestamp.into()),
    )
    .unwrap()
}

fn bitvec_to_bytes(bitvec: BitVec) -> Vec<u8> {
    let mut bitvec_as_u8_array = vec![0; (bitvec.len() / 8) + 1];

    for (idx, bit) in bitvec.into_iter().enumerate() {
        let byte = idx / 8;
        let shift = 7 - idx % 8;
        bitvec_as_u8_array[byte] |= (bit as u8) << shift;
    }

    bitvec_as_u8_array
}

fn bytes_to_bitvec(bytes: Vec<u8>) -> BitVec {
    let rev_list: BitVec<_> = BitVec::from_vec(bytes);
    rev_list.into_iter().collect()
}

fn hex_to_bytes(hex_str: &str) -> Vec<u8> {
    let hex_str = hex_str.trim_start_matches("0x");
    hex::decode(hex_str).unwrap()
}
