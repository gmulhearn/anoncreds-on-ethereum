use std::sync::Arc;

use anoncreds::data_types::rev_status_list::serde_revocation_list;
use ethers::{
    prelude::{abigen, k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, H160},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use uuid::Uuid;

// generate type-safe bindings to contract with ethers-rs
abigen!(
    AnoncredsRegistryContract,
    "../anoncreds-smart-contracts-js/artifacts/contracts/AnoncredsRegistry.sol/AnoncredsRegistry.json"
);

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
        let did = address_as_did(&self.author_pub_key);
        format!("{}/{}", did, self.resource_path)
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
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

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
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

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

pub async fn submit_rev_reg_status_update(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    rev_reg_id: &str,
    revocation_status_list: &anoncreds::types::RevocationStatusList,
) {
    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

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

    let status_list = anoncreds_registry_contract::RevocationStatusList {
        revocation_list: serialized_bitvec_revocation_list,
        current_accumulator,
    };

    contract
        .add_rev_reg_status_update(String::from(rev_reg_id), status_list)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
}
