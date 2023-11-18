use std::env;
use std::error::Error;
use std::sync::Arc;

use anoncreds::data_types::rev_status_list::serde_revocation_list;
use anyhow::anyhow;
use bitvec::{prelude::Lsb0, vec::BitVec};
use dotenv::dotenv;
use ethers::contract::EthLogDecode;
use ethers::providers::Middleware;
use ethers::signers::coins_bip39::English;
use ethers::signers::{MnemonicBuilder, Signer};
use ethers::types::Address;
use ethers::{
    abi::RawLog,
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::{H160, U256},
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
pub const ANONCRED_REGISTRY_ADDRESS: &str = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";

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
    AnoncredsRegistry::new(
        ANONCRED_REGISTRY_ADDRESS.parse::<Address>().unwrap(),
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
    /// The immutable resource is published using the signer author held by this
    /// [AnoncredsEthRegistry] instance. The resource is given a random ID, under the
    /// provided [parent_path].
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
    pub async fn submit_rev_reg_status_update(
        &self,
        signer: Arc<impl Middleware>,
        did: &str,
        rev_reg_id: &str,
        revocation_status_list: &anoncreds::types::RevocationStatusList,
    ) -> u64 {
        let contract = contract_with_client(signer);

        let did_identity = full_did_into_did_identity(did);

        // dismantle the inner parts that we want to upload to the registry
        let revocation_status_list_json: Value =
            serde_json::to_value(revocation_status_list).unwrap();
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
            .add_revocation_registry_status_update(
                did_identity,
                String::from(rev_reg_id),
                status_list,
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();

        // extract the emitted [AnoncredsRegistryEvents::NewRevRegStatusUpdateFilter] event
        // from the transaction's receipt. This event importantly contains the ledger
        // timestamp that will be used for that revocation status list entry.
        let mut eth_events = tx
            .logs
            .into_iter()
            .filter_map(|log| AnoncredsRegistryEvents::decode_log(&RawLog::from(log)).ok());

        let rev_reg_update_event = eth_events
            .find_map(|log| match log {
                AnoncredsRegistryEvents::NewRevRegStatusUpdateFilter(inner) => Some(inner),
                _ => None,
            })
            .unwrap();

        let ledger_recorded_timestamp = rev_reg_update_event.timestamp as u64;

        ledger_recorded_timestamp
    }

    /// For the given [rev_reg_id], find the revocation status list entry that is
    /// closest to the given [timestamp], but no later.
    ///
    /// Returns the closest revocation status list, and the timestamp of that revocation
    /// status list entry. (The timestamp is also within the status list object, but is
    /// not accessible).
    pub async fn get_rev_reg_status_list_as_of_timestamp(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> (anoncreds::types::RevocationStatusList, u64) {
        let client = get_read_only_ethers_client();
        let contract = contract_with_client(client);

        let rev_reg_resource_id = DIDResourceId::from_id(rev_reg_id.to_owned());

        // get the timestamps for all revocation status list updates that have been made.
        let all_timestamps: Vec<u32> = contract
            .get_revocation_registry_update_timestamps(
                rev_reg_resource_id.did_identity,
                String::from(rev_reg_id),
            )
            .call()
            .await
            .unwrap();
        let all_timestamps: Vec<u64> = all_timestamps.into_iter().map(u64::from).collect();

        if all_timestamps.is_empty() {
            panic!("No rev entries for rev reg: {rev_reg_id}")
        }

        // TODO - here we might binary search rather than iter all
        // Find the index of the timestamp that is closest to the provided [timestamp]:
        // * scan the list until a timestamp is greater than the desired [timestamp],
        //  then minus one from the index of that item.
        // * OR, if no entries are greater than, just pick the last/latest entry.
        let index_of_entry = all_timestamps
            .iter()
            .position(|ts| ts > &timestamp)
            .unwrap_or(all_timestamps.len())
            - 1;
        let timestamp_of_entry = all_timestamps[index_of_entry];

        // get the revocation status list information of the determined index from the registry.
        let entry: anoncreds_registry::RevocationStatusList = contract
            .get_revocation_registry_update_at_index(
                rev_reg_resource_id.did_identity,
                String::from(rev_reg_id),
                U256::from(index_of_entry),
            )
            .call()
            .await
            .unwrap();

        // reconstruct the [anoncreds::types::RevocationStatusList] from the registry stored
        // data.
        let mut rev_list: BitVec = serde_json::from_str(&entry.revocation_list).unwrap();
        let mut recapacitied_rev_list = BitVec::<usize, Lsb0>::with_capacity(64);
        recapacitied_rev_list.append(&mut rev_list);

        let current_accumulator =
            serde_json::from_value(json!(&entry.current_accumulator)).unwrap();

        let rev_list = anoncreds::types::RevocationStatusList::new(
            Some(rev_reg_id),
            rev_reg_resource_id.author_did().try_into().unwrap(),
            recapacitied_rev_list,
            Some(current_accumulator),
            Some(timestamp_of_entry),
        )
        .unwrap();

        (rev_list, timestamp_of_entry)
    }
}
