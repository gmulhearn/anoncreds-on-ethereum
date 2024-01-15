use std::{env, sync::Arc};

use dotenv::dotenv;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet},
};

// Ethereum RPC of the network to use (defaults to the hardhat local network)
const REGISTRY_RPC: &str = "http://localhost:8545";

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
