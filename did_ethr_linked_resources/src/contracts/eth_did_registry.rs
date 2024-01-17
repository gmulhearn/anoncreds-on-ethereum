use std::sync::Arc;

use ethers::{abi::Address, providers::Middleware, types::H160};

use crate::{config::ContractNetworkConfig, utils::full_did_into_did_identity};

// Include generated contract types from build script
include!(concat!(
    env!("OUT_DIR"),
    "/ethereum_did_registry_contract.rs"
));

pub struct DidEthRegistry {
    contract_address: Address,
    _rpc_url: String,
}

impl DidEthRegistry {
    pub fn new(config: ContractNetworkConfig) -> Self {
        Self {
            contract_address: config.contract_address.parse().unwrap(),
           _rpc_url: config.rpc_url,
        }
    }

    fn contract_with_client<T: Middleware>(&self, client: Arc<T>) -> EthereumDIDRegistry<T> {
        EthereumDIDRegistry::new(self.contract_address.clone(), client)
    }

    pub async fn change_owner(&self, signer: Arc<impl Middleware>, did: &str, new_owner: H160) {
        let contract = self.contract_with_client(signer);

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
