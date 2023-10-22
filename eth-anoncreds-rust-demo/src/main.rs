pub mod anoncreds_eth_registry;
pub mod roles;
pub mod utils;

use std::time::Duration;
use tokio::time::sleep;

use crate::{
    anoncreds_eth_registry::get_default_ethers_client,
    roles::{CredRevocationUpdateType, Holder, Issuer, Verifier},
    utils::get_epoch_secs,
};

#[tokio::main]
async fn main() {
    full_demo().await
}

async fn full_demo() {
    // ------ SETUP CLIENTS ------
    let signer = get_default_ethers_client();

    println!("Holder: setting up...");
    let mut holder = Holder::bootstrap(signer.clone()).await;
    println!("Issuer: setting up...");
    let mut issuer = Issuer::bootstrap(signer.clone()).await;
    println!("Verifier: setting up...");
    let mut verifier = Verifier::bootstrap(signer.clone());

    // issue the cred to the holder
    issuance_demo(&mut holder, &mut issuer).await;

    // present without a NRP
    let mut prover = holder;
    assert!(presentation_demo(&mut prover, &mut verifier).await);

    // present with a NRP
    let first_epoch = get_epoch_secs();
    assert!(presentation_demo_with_nrp(&mut prover, &mut verifier, first_epoch).await);

    // revoke the holder's cred
    issuer
        .update_credential_revocation(CredRevocationUpdateType::Revoke)
        .await;

    sleep(Duration::from_secs(3)).await;

    let second_epoch = get_epoch_secs();
    // cannot present validly for newer interval
    assert!(!presentation_demo_with_nrp(&mut prover, &mut verifier, second_epoch).await);
    // can present validly for older interval still
    assert!(presentation_demo_with_nrp(&mut prover, &mut verifier, first_epoch).await);

    // unrevoke the holder's cred
    issuer
        .update_credential_revocation(CredRevocationUpdateType::Issue)
        .await;

    sleep(Duration::from_secs(3)).await;
    let third_epoch = get_epoch_secs();
    // can present validly for newer interval
    assert!(presentation_demo_with_nrp(&mut prover, &mut verifier, third_epoch).await);
}

/// Run thru a single credential issuance flow. Issuing a revocable credential.
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

/// Run thru a single proof presentation, with a cred_def restriction.
async fn presentation_demo(prover: &mut Holder, verifier: &mut Verifier) -> bool {
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

    valid
}

/// Run thru a single proof presentation, with a cred_def restriction, and request a NRP
/// with an interval of `non_revoked_as_of`.
async fn presentation_demo_with_nrp(
    prover: &mut Holder,
    verifier: &mut Verifier,
    non_revoked_as_of: u64,
) -> bool {
    println!("\n########## PRESENTATION ###########\n");

    println!("Verifier: Creating NRP presentation request for interval '..{non_revoked_as_of}'");
    let from_cred_def = &prover.get_credential().cred_def_id.0;
    let pres_req = verifier.request_presentation_with_nrp(from_cred_def, non_revoked_as_of);

    println!("Prover: creating presentation...");
    let presentation = prover.present_credential_with_nrp(&pres_req).await;

    println!("Verifier: verifying prover's presentation...");
    let valid = verifier.verify_presentation_with_nrp(&presentation).await;
    println!("Verifier: verified presentation... Verified presentation: {valid}");

    println!("\n########## END OF PRESENTATION ###########\n");

    valid
}
