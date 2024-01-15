use std::sync::Arc;

use ethers::{abi::Address, providers::Middleware, types::H160};

use crate::ledger::did_parsing_helpers::full_did_into_did_identity;

// Include generated contract types from build script
include!(concat!(
    env!("OUT_DIR"),
    "/ethereum_did_registry_contract.rs"
));

// Address of the `EthereumDIDRegistry` smart contract to use
// (should copy and paste the address value after a hardhat deploy script)
pub const DID_REGISTRY_ADDRESS: &str = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

pub fn contract_with_client<T: Middleware>(client: Arc<T>) -> EthereumDIDRegistry<T> {
    EthereumDIDRegistry::new(DID_REGISTRY_ADDRESS.parse::<Address>().unwrap(), client)
}

pub struct DidEthRegistry;

impl DidEthRegistry {
    pub async fn change_owner(&self, signer: Arc<impl Middleware>, did: &str, new_owner: H160) {
        let contract = contract_with_client(signer);

        let did_identity = full_did_into_did_identity(did);

        contract
            .change_owner(did_identity, new_owner)
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }
}
