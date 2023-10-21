use std::sync::Arc;

use anoncreds::{
    data_types::{
        cred_def::CredentialDefinition, rev_status_list::serde_revocation_list, schema::Schema,
    },
    types::RevocationRegistryDefinition,
};
use ethers::{
    prelude::{abigen, k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, H160},
};
use serde_json::Value;

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
pub struct SchemaIdParts {
    pub issuer_pub_key: H160,
    pub name: String,
    pub version: String,
}

impl SchemaIdParts {
    pub fn from_id(schema_id: String) -> Self {
        let parts: Vec<&str> = schema_id.split(":").collect();

        let issuer = parts[2];
        let issuer_pub_key = issuer.parse().unwrap();

        let name = String::from(parts[4]);

        let version = String::from(parts[5]);

        SchemaIdParts {
            issuer_pub_key,
            name,
            version,
        }
    }

    pub fn to_id(&self) -> String {
        let did = address_as_did(&self.issuer_pub_key);
        format!("{}:schema:{}:{}", did, self.name, self.version)
    }
}

pub async fn submit_schema(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    schema: &Schema,
) -> SchemaIdParts {
    let schema_json = serde_json::to_string(schema).unwrap();

    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

    let name = schema.name.clone();
    let version = schema.version.clone();

    contract
        .create_schema(name.clone(), version.clone(), schema_json)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    SchemaIdParts {
        issuer_pub_key: client.address(),
        name,
        version,
    }
}

pub async fn get_schema(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    schema_id: &str,
) -> Schema {
    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

    let schema_id_parts = SchemaIdParts::from_id(schema_id.to_owned());

    let schema_json: String = contract
        .get_schema(
            schema_id_parts.issuer_pub_key,
            schema_id_parts.name,
            schema_id_parts.version,
        )
        .call()
        .await
        .unwrap();

    serde_json::from_str(&schema_json).unwrap()
}

#[derive(Debug)]
pub struct CredDefIdParts {
    pub issuer_pub_key: H160,
    pub tag: String,
    pub schema_id: String,
}

impl CredDefIdParts {
    // super panicy!
    pub fn from_id(cred_def_id: String) -> Self {
        let parts: Vec<&str> = cred_def_id.split(":").collect();

        let issuer = parts[2];
        let issuer_pub_key = issuer.parse().unwrap();

        let tag = String::from(parts[4]);

        // everything else, == schema_id
        let schema_id = parts[5..].join(":");

        CredDefIdParts {
            issuer_pub_key,
            tag,
            schema_id,
        }
    }

    pub fn to_id(&self) -> String {
        let did = address_as_did(&self.issuer_pub_key);
        format!("{}:creddef:{}:{}", did, self.tag, self.schema_id)
    }
}

pub async fn submit_cred_def(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    cred_def: &CredentialDefinition,
) -> CredDefIdParts {
    let cred_def_json = serde_json::to_string(cred_def).unwrap();

    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

    let schema_id = cred_def.schema_id.0.clone();
    let tag = cred_def.tag.clone();

    contract
        .create_cred_def(schema_id.clone(), tag.clone(), cred_def_json)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    CredDefIdParts {
        issuer_pub_key: client.address(),
        tag,
        schema_id,
    }
}

pub async fn get_cred_def(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    cred_def_id: &str,
) -> CredentialDefinition {
    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

    let cred_def_parts = CredDefIdParts::from_id(cred_def_id.to_owned());

    let cred_def_json: String = contract
        .get_cred_def(
            cred_def_parts.issuer_pub_key,
            cred_def_parts.schema_id,
            cred_def_parts.tag,
        )
        .call()
        .await
        .unwrap();

    serde_json::from_str(&cred_def_json).unwrap()
}

#[derive(Debug)]
pub struct RevRegIdParts {
    pub issuer_pub_key: H160,
    pub tag: String,
    pub cred_def_id: String,
}

impl RevRegIdParts {
    // super panicy!
    pub fn from_id(rev_reg_id: String) -> Self {
        let parts: Vec<&str> = rev_reg_id.split(":").collect();

        let issuer = parts[2];
        let issuer_pub_key = issuer.parse().unwrap();

        let tag = String::from(parts[4]);

        // everything else, == cred_def_id
        let cred_def_id = parts[5..].join(":");

        RevRegIdParts {
            issuer_pub_key,
            tag,
            cred_def_id,
        }
    }

    pub fn to_id(&self) -> String {
        let did = address_as_did(&self.issuer_pub_key);
        format!("{}:rev_reg:{}:{}", did, self.tag, self.cred_def_id)
    }
}

pub async fn submit_rev_reg_def(
    client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    rev_reg_def: &RevocationRegistryDefinition,
) -> RevRegIdParts {
    let rev_reg_def_json = serde_json::to_string(rev_reg_def).unwrap();

    let address: Address = REGISTRY_WETH_ADDRESS.parse().unwrap();
    let contract = AnoncredsRegistryContract::new(address, Arc::clone(client));

    let cred_def_id = rev_reg_def.cred_def_id.0.clone();
    let tag = rev_reg_def.tag.clone();

    contract
        .create_rev_reg_def(cred_def_id.clone(), tag.clone(), rev_reg_def_json)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    RevRegIdParts {
        issuer_pub_key: client.address(),
        tag,
        cred_def_id,
    }
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
