pub mod anoncreds_eth_registry;
pub mod roles;
pub mod utils;

use std::{env, sync::Arc};

use anoncreds_eth_registry::REGISTRY_RPC;
use dotenv::dotenv;
use ethers::{
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet},
};

use crate::roles::{Holder, Issuer, Verifier};

pub type EtherSigner = Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;

fn get_ethers_client() -> Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    dotenv().ok();

    let seed = env::var("MNEMONIC").unwrap();

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(&*seed)
        .build()
        .unwrap()
        .with_chain_id(31337 as u64);

    let provider = Provider::<Http>::try_from(REGISTRY_RPC).unwrap();
    let x = Arc::new(SignerMiddleware::new(provider, wallet));

    x
}

#[tokio::main]
async fn main() {
    full_demo().await
}

async fn full_demo() {
    // ------ SETUP CLIENTS ------
    let signer = get_ethers_client();

    println!("Holder: setting up...");
    let mut holder = Holder::bootstrap(signer.clone()).await;
    println!("Issuer: setting up...");
    let mut issuer = Issuer::bootstrap(signer.clone()).await;
    println!("Verifier: setting up...");
    let mut verifier = Verifier::bootstrap(signer.clone());

    issuance_demo(&mut holder, &mut issuer).await;

    let mut prover = holder;
    presentation_demo_with_nrp(&mut prover, &mut verifier).await;
    // presentation_demo(&mut prover, &mut verifier).await;

    // issuer.revoke_credential().await;
}

async fn issuance_demo(holder: &mut Holder, issuer: &mut Issuer) {
    println!("\n########## ISSUANCE ###########\n");
    println!("Issuer: creating credential offer...");
    let offer = issuer.create_offer();

    println!("Holder: creating credential request from offer...");
    let request = holder.accept_offer(&offer).await;

    println!("Issuer: issuing credential for holder's request...");
    let issued_cred = issuer.create_credential(&request, "John Smith", "28");

    println!("Holder: storing credential from issuer...");
    holder.store_credential(issued_cred).await;

    println!(
        "Holder: Awwww yea, check out my creds: {:?}",
        holder.get_credential()
    );
    println!("\n########## END OF ISSUANCE ###########\n");
}

async fn _presentation_demo(prover: &mut Holder, verifier: &mut Verifier) {
    println!("\n########## PRESENTATION ###########\n");

    println!("Verifier: Creating presentation request...");
    let from_cred_def = &prover.get_credential().cred_def_id.0;
    let pres_req = verifier.request_presentation(from_cred_def);

    println!("Prover: creating presentation...");
    let presentation = prover.present_credential(&pres_req).await;

    println!("Verifier: verifying prover's presentation...");
    let valid = verifier.verify_presentation(&presentation).await;
    println!("Verifier: verified presentation... Verified presentation: {valid}");

    println!("\n########## END OF PRESENTATION ###########\n");
}

async fn presentation_demo_with_nrp(prover: &mut Holder, verifier: &mut Verifier) {
    println!("\n########## PRESENTATION ###########\n");

    println!("Verifier: Creating presentation request...");
    let from_cred_def = &prover.get_credential().cred_def_id.0;
    let pres_req = verifier.request_presentation_with_nrp(from_cred_def);

    println!("Prover: creating presentation...");
    let presentation = prover.present_credential_with_nrp(&pres_req).await;

    println!("Verifier: verifying prover's presentation...");
    let valid = verifier.verify_presentation_with_nrp(&presentation).await;
    println!("Verifier: verified presentation... Verified presentation: {valid}");

    println!("\n########## END OF PRESENTATION ###########\n");
}
