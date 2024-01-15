use std::sync::Arc;

use ethers::providers::{Http, Middleware, Provider};

pub mod eth_did_registry;
pub mod ethr_dlr_registry;

// Ethereum RPC of the network to use (defaults to the hardhat local network)
pub const REGISTRY_RPC: &str = "http://localhost:8545";

pub fn get_read_only_ethers_client() -> Arc<impl Middleware> {
    let provider = Provider::<Http>::try_from(REGISTRY_RPC).unwrap();
    Arc::new(provider)
}

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

    use super::REGISTRY_RPC;
    pub fn get_writer_ethers_client(
        id: u32,
    ) -> Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
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
}
