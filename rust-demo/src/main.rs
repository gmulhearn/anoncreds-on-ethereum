pub mod anoncreds_eth_registry;

use std::{collections::HashMap, env, sync::Arc};

use anoncreds::{
    data_types::{
        cred_def::{CredentialDefinition, CredentialDefinitionId},
        schema::{Schema, SchemaId},
    },
    prover::create_presentation,
    types::{
        Credential, CredentialDefinitionConfig, CredentialDefinitionPrivate,
        CredentialKeyCorrectnessProof, LinkSecret, MakeCredentialValues, PresentCredentials,
        PresentationRequest, SignatureType,
    },
};
use anoncreds_eth_registry::{
    address_as_did, get_cred_def, get_schema, submit_cred_def, submit_schema, CredDefIdParts,
    SchemaIdParts, REGISTRY_RPC,
};
use dotenv::dotenv;
use ethers::{
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet},
};

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
    println!("Holder: setting up...");
    let holder = get_ethers_client();
    let holder_link_secret = anoncreds::prover::create_link_secret().unwrap();
    let holder_link_secret_id = "1";

    println!("Issuer: setting up...");
    let issuer = get_ethers_client();

    println!("\n########## ISSUANCE ###########\n");

    // ---- ISSUER -----
    // bootstrap data onto ledger
    println!("Issuer: setting up ledger data...");
    let (
        schema_id_parts,
        cred_def_id_parts,
        _schema,
        cred_def,
        cred_def_private,
        correctness_proof,
    ) = issuer_bootstrap_ledger_submissions(&issuer).await;
    let schema_id = schema_id_parts.to_id();
    let cred_def_id = cred_def_id_parts.to_id();
    println!(
        "Issuer: ledger data created. \n\tSchema ID: {schema_id}. \n\tCred Def ID: {cred_def_id}"
    );

    // create offer for holder
    println!("Issuer: creating credential offer...");
    let cred_offer = anoncreds::issuer::create_credential_offer(
        schema_id.clone(),
        cred_def_id.clone(),
        &correctness_proof,
    )
    .unwrap();

    //... send over didcomm...

    // ---- HOLDER -----
    // fetch cred def from ledger based on offer's ID
    println!("Holder: fetching cred def from offer...");
    let holder_fetched_cred_def = get_cred_def(&holder, &cred_offer.cred_def_id.0).await;
    // create request in response to offer
    println!("Holder: creating credential request from offer and cred def...");
    let (cred_request, cred_request_metadata) = anoncreds::prover::create_credential_request(
        Some(&uuid::Uuid::new_v4().to_string()),
        None,
        &holder_fetched_cred_def,
        &holder_link_secret,
        &holder_link_secret_id,
        &cred_offer,
    )
    .unwrap();

    //... send over didcomm...

    // ---- ISSUER ----
    // issue out the credential!
    println!("Issuer: issuing credential for holder's request...");
    let mut credential_values = MakeCredentialValues::default();
    credential_values.add_raw("name", "john").unwrap();
    credential_values.add_raw("age", "28").unwrap();
    let issued_credential = anoncreds::issuer::create_credential(
        &cred_def,
        &cred_def_private,
        &cred_offer,
        &cred_request,
        credential_values.into(),
        None,
        None,
        None,
    )
    .unwrap();

    //... send over didcomm...

    // ---- HOLDER ----
    // store cred (in memory after processing it)
    println!("Holder: storing credential from issuer...");
    let mut holders_stored_credential = issued_credential.try_clone().unwrap();
    anoncreds::prover::process_credential(
        &mut holders_stored_credential,
        &cred_request_metadata,
        &holder_link_secret,
        &holder_fetched_cred_def,
        None,
    )
    .unwrap();
    println!("Holder: Awwww yea, check out my creds: {holders_stored_credential:?}");

    println!("\n########## END OF ISSUANCE ###########\n");

    // now lets present...
    proof_presentation_demo(
        &issuer,
        &holder,
        cred_def_id,
        holder_link_secret,
        holders_stored_credential,
    )
    .await
}

async fn proof_presentation_demo(
    _verifier_client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    holder_client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    verifier_cred_def_id: String,
    holder_link_secret: LinkSecret,
    holder_cred: Credential,
) {
    println!("\n########## PRESENTATION ###########\n");

    // --- VERIFIER ---
    println!("Verifier: Creating presentation request...");
    let nonce = anoncreds::verifier::generate_nonce().unwrap();
    let pres_req: PresentationRequest = serde_json::from_value(serde_json::json!({
        "nonce": nonce,
        "name":"example_presentation_request",
        "version":"0.1",
        "requested_attributes":{
            "reft1":{
                "name":"age",
                "restrictions": {
                    "cred_def_id": verifier_cred_def_id
                }
            },
        },
    }))
    .unwrap();

    //... send over didcomm...

    // --- PROVER ----
    println!("Prover: constructing data for proof...");

    // construct data from ledger
    // construct schemas
    println!("Prover: fetching schema for my credential from ledger...");
    let mut schemas: HashMap<&SchemaId, &Schema> = HashMap::new();
    let schema_for_cred = get_schema(holder_client, &holder_cred.schema_id.0).await;
    schemas.insert(&holder_cred.schema_id, &schema_for_cred);

    // construct cred defs
    println!("Prover: fetching cred def for my credential from ledger...");
    let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
    let cred_def_for_cred = get_cred_def(holder_client, &holder_cred.cred_def_id.0).await;
    cred_defs.insert(&holder_cred.cred_def_id, &cred_def_for_cred);

    // specify creds to use for referents
    let mut creds_to_present = PresentCredentials::default();
    let mut added_cred = creds_to_present.add_credential(&holder_cred, None, None);
    added_cred.add_requested_attribute("reft1", true);

    println!("Prover: creating presentation...");
    let presentation = create_presentation(
        &pres_req,
        creds_to_present,
        None,
        &holder_link_secret,
        &schemas,
        &cred_defs,
    )
    .unwrap();

    //... send over didcomm...

    // --- VERIFIER ----
    // construct data from ledger
    // technically the verifier should be fetching from the ledger here to
    // construct `schemas` and `cred_defs`... but you get the point, so we just reuse the holder's data.
    println!("Verifier: fetching schema for presented identifier from ledger...");
    let schemas = schemas;
    println!("Verifier: fetching cred def for presented identifier from ledger...");
    let cred_defs = cred_defs;

    println!("Verifier: verifying prover's presentation...");
    let valid = anoncreds::verifier::verify_presentation(
        &presentation,
        &pres_req,
        &schemas,
        &cred_defs,
        None,
        None,
        None,
    )
    .unwrap();
    println!("Verifier: verified presentation... Verified presentation: {valid}");

    println!("\n########## END OF PRESENTATION ###########\n");
}

async fn issuer_bootstrap_ledger_submissions(
    issuer_client: &Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> (
    SchemaIdParts,
    CredDefIdParts,
    Schema,
    CredentialDefinition,
    CredentialDefinitionPrivate,
    CredentialKeyCorrectnessProof,
) {
    let signer_address = issuer_client.address();
    let issuer_id = address_as_did(&signer_address);

    let attr_names: &[&str] = &["name", "age"];

    let schema_name = format!("ChadCertification-{}", uuid::Uuid::new_v4().to_string());
    println!("Issuer: creating schema for schema name: {schema_name}...");
    let schema =
        anoncreds::issuer::create_schema(&schema_name, "1.0", issuer_id.clone(), attr_names.into())
            .unwrap();

    // upload to ledger
    println!("Issuer: submitting schema...");
    let schema_id_parts = submit_schema(&issuer_client, &schema).await;
    let schema_id = schema_id_parts.to_id();

    let cred_def_tag = format!(
        "BasedChadCertification-{}",
        uuid::Uuid::new_v4().to_string()
    );
    println!("Issuer: creating cred def for tag: {cred_def_tag}...");
    let (cred_def, cred_def_private, correctness_proof) =
        anoncreds::issuer::create_credential_definition(
            schema_id.clone(),
            &schema,
            issuer_id.clone(),
            &cred_def_tag,
            SignatureType::CL,
            CredentialDefinitionConfig::new(false),
        )
        .unwrap();

    // upload to ledger
    println!("Issuer: submitting cred def...");
    let cred_def_id_parts = submit_cred_def(&issuer_client, &cred_def).await;

    (
        schema_id_parts,
        cred_def_id_parts,
        schema,
        cred_def,
        cred_def_private,
        correctness_proof,
    )
}
