use std::{env, sync::Arc};

use dotenv::dotenv;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet},
};

use crate::config::DemoConfig;

pub type EtherSigner = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

pub fn get_writer_ethers_client(id: u32, config: &DemoConfig) -> Arc<EtherSigner> {
    dotenv().ok();

    let seed = env::var("MNEMONIC").unwrap();

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(&*seed)
        .index(id)
        .unwrap()
        .build()
        .unwrap()
        .with_chain_id(config.chain_id);

    let provider = Provider::<Http>::try_from(&config.rpc_url).unwrap();
    Arc::new(SignerMiddleware::new(provider, wallet))
}
