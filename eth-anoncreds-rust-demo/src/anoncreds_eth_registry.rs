use std::sync::Arc;

use anoncreds::data_types::{cred_def::CredentialDefinition, schema::Schema};
use ethers::{
    prelude::{abigen, k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, H160},
};

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
    pub issuer: H160,
    pub name: String,
    pub version: String,
}

impl SchemaIdParts {
    pub fn from_id(schema_id: String) -> Self {
        let parts: Vec<&str> = schema_id.split(":").collect();

        let issuer = parts[2];
        let issuer = issuer.parse().unwrap();

        let name = String::from(parts[4]);

        let version = String::from(parts[5]);

        SchemaIdParts {
            issuer,
            name,
            version,
        }
    }

    pub fn to_id(&self) -> String {
        let did = address_as_did(&self.issuer);
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
        issuer: client.address(),
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
            schema_id_parts.issuer,
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
    pub issuer: H160,
    pub tag: String,
    pub schema_id: String,
}

impl CredDefIdParts {
    // super panicy!
    pub fn from_id(cred_def_id: String) -> Self {
        let parts: Vec<&str> = cred_def_id.split(":").collect();

        let issuer = parts[2];
        let issuer = issuer.parse().unwrap();

        let tag = String::from(parts[4]);

        // everything else, == schema_id
        let schema_id = parts[5..].join(":");

        CredDefIdParts {
            issuer,
            tag,
            schema_id,
        }
    }

    pub fn to_id(&self) -> String {
        let did = address_as_did(&self.issuer);
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
        issuer: client.address(),
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
            cred_def_parts.issuer,
            cred_def_parts.schema_id,
            cred_def_parts.tag,
        )
        .call()
        .await
        .unwrap();

    serde_json::from_str(&cred_def_json).unwrap()
}
