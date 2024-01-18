pub mod eth_did_registry;
pub mod ethr_dlr_registry;

#[cfg(test)]
pub mod test_utils {
    use std::{env, sync::Arc};

    use dotenv::dotenv;
    use ethers::{
        core::k256::ecdsa::SigningKey,
        middleware::SignerMiddleware,
        providers::{Http, Provider},
        signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet},
    };

    use crate::config::ContractNetworkConfig;

    pub struct TestConfig {
        pub rpc_url: String,
        pub dlr_contract_address: String,
        pub chain_id: u64,
    }

    impl TestConfig {
        pub fn load() -> Self {
            dotenv().ok();

            let rpc_url = env::var("RPC_URL").unwrap();
            let dlr_contract_address = env::var("DLR_CONTRACT_ADDRESS").unwrap();
            let chain_id = env::var("CHAIN_ID").unwrap().parse().unwrap();

            Self {
                rpc_url,
                dlr_contract_address,
                chain_id,
            }
        }

        pub fn get_dlr_network_config(&self) -> ContractNetworkConfig {
            ContractNetworkConfig {
                rpc_url: self.rpc_url.clone(),
                contract_address: self.dlr_contract_address.clone(),
                chain_id: self.chain_id,
            }
        }
    }

    pub fn get_writer_ethers_client(
        id: u32,
        conf: &TestConfig,
    ) -> Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
        dotenv().ok();

        let seed = env::var("MNEMONIC").unwrap();

        let wallet = MnemonicBuilder::<English>::default()
            .phrase(&*seed)
            .index(id)
            .unwrap()
            .build()
            .unwrap()
            .with_chain_id(conf.chain_id);

        let provider = Provider::<Http>::try_from(&conf.rpc_url).unwrap();
        Arc::new(SignerMiddleware::new(provider, wallet))
    }
}
