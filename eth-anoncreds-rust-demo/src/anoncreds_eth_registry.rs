use std::sync::Arc;

use anoncreds::data_types::rev_status_list::serde_revocation_list;
use ethers::contract::EthLogDecode;
use ethers::{
    abi::RawLog,
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, H160, U256},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

// Include generated contract types from build script
include!(concat!(env!("OUT_DIR"), "/anoncreds_registry_contract.rs"));

// Ethereum RPC of the network to use (defaults to the hardhat local network)
pub const REGISTRY_RPC: &str = "http://localhost:8545";
// Address of the `AnoncredsRegistry` smart contract to use
// (should copy and paste the address value after a hardhat deploy script)
pub const REGISTRY_WETH_ADDRESS: &str = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

// Hacked up DID method for ethereum addresses (probably not 100% valid)
pub fn address_as_did(address: &H160) -> String {
    // note that debug fmt of address is the '0x..' hex encoding.
    // where as .to_string() (fmt) truncates it
    format!("did:based:{address:?}")
}

#[derive(Debug)]
pub struct DIDResourceId {
    pub author_pub_key: H160,
    pub resource_path: String,
}

impl DIDResourceId {
    pub fn from_id(id: String) -> Self {
        let Some((did, resource_path)) = id.split_once("/") else {
            panic!("Could not process as DID Resource: {id}")
        };

        let author = did
            .split(":")
            .last()
            .expect(&format!("Could not read find author of DID: {did}"));
        let author_pub_key = author.parse().unwrap();

        DIDResourceId {
            author_pub_key,
            resource_path: resource_path.to_owned(),
        }
    }

    pub fn to_id(&self) -> String {
        let did = self.author_did();
        format!("{}/{}", did, self.resource_path)
    }

    pub fn author_did(&self) -> String {
        address_as_did(&self.author_pub_key)
    }
}

pub async fn submit_json_resource<S>(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    resource: &S,
    parent_path: &str,
) -> DIDResourceId
where
    S: Serialize,
{
    let resource_json = serde_json::to_string(resource).unwrap();
    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();

    let contract = AnoncredsRegistry::new(address, Arc::clone(client));

    let resource_path = format!("{parent_path}/{}", Uuid::new_v4());

    contract
        .create_immutable_resource(resource_path.clone(), resource_json)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    DIDResourceId {
        author_pub_key: client.address(),
        resource_path,
    }
}

pub async fn get_json_resource<D>(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    resource_id: &str,
) -> D
where
    D: DeserializeOwned,
{
    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistry::new(address, Arc::clone(client));

    let did_resource_parts = DIDResourceId::from_id(resource_id.to_owned());

    let resource_json: String = contract
        .get_immutable_resource(
            did_resource_parts.author_pub_key,
            did_resource_parts.resource_path,
        )
        .call()
        .await
        .unwrap();

    serde_json::from_str(&resource_json).unwrap()
}

/// Returns the real timestamp recorded on the ledger
pub async fn submit_rev_reg_status_update(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    rev_reg_id: &str,
    revocation_status_list: &anoncreds::types::RevocationStatusList,
) -> u64 {
    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistry::new(address, Arc::clone(client));

    let revocation_status_list_json: Value = serde_json::to_value(revocation_status_list).unwrap();
    let current_accumulator = revocation_status_list_json
        .get("currentAccumulator")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    let revocation_list_val = revocation_status_list_json.get("revocationList").unwrap();
    let bitvec = serde_revocation_list::deserialize(revocation_list_val).unwrap();
    let serialized_bitvec_revocation_list = serde_json::to_string(&bitvec).unwrap();

    let status_list = anoncreds_registry::RevocationStatusList {
        revocation_list: serialized_bitvec_revocation_list,
        current_accumulator,
    };

    let tx = contract
        .add_rev_reg_status_update(String::from(rev_reg_id), status_list)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let mut eth_events = tx
        .logs
        .into_iter()
        .map(|log| AnoncredsRegistryEvents::decode_log(&RawLog::from(log)).unwrap());

    let rev_reg_update_event = eth_events
        .find_map(|log| match log {
            AnoncredsRegistryEvents::NewRevRegStatusUpdateFilter(inner) => Some(inner),
            _ => None,
        })
        .unwrap();

    dbg!(&rev_reg_update_event);

    let ledger_recorded_timestamp = rev_reg_update_event.timestamp.as_u64();

    ledger_recorded_timestamp
}

pub async fn get_rev_reg_status_list_as_of_timestamp(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    rev_reg_id: &str,
    timestamp: u64,
) -> (anoncreds::types::RevocationStatusList, u64) {
    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistry::new(address, Arc::clone(client));

    let rev_reg_resource_id = DIDResourceId::from_id(rev_reg_id.to_owned());

    let all_timestamps: Vec<U256> = contract
        .get_rev_reg_update_timestamps(rev_reg_resource_id.author_pub_key, String::from(rev_reg_id))
        .call()
        .await
        .unwrap();

    if all_timestamps.is_empty() {
        panic!("No rev entries for rev reg: {rev_reg_id}")
    }

    // TODO - here we might binary search rather than iter all
    let index_of_entry = all_timestamps
        .iter()
        .position(|ts| ts > &U256::from(timestamp))
        .unwrap_or(all_timestamps.len())
        - 1;
    let timestamp_of_entry = all_timestamps[index_of_entry].as_u64();

    let entry: anoncreds_registry::RevocationStatusList = contract
        .get_rev_reg_update_at_index(
            rev_reg_resource_id.author_pub_key,
            String::from(rev_reg_id),
            U256::from(index_of_entry),
        )
        .call()
        .await
        .unwrap();

    let rev_list = serde_json::from_str(&entry.revocation_list).unwrap();
    let current_accumulator = serde_json::from_value(json!(&entry.current_accumulator)).unwrap();

    let rev_list = anoncreds::types::RevocationStatusList::new(
        Some(rev_reg_id),
        rev_reg_resource_id.author_did().try_into().unwrap(),
        rev_list,
        Some(current_accumulator),
        Some(timestamp_of_entry),
    )
    .unwrap();
    (rev_list, timestamp_of_entry)
}
