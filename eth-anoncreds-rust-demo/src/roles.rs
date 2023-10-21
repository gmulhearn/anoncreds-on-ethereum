use std::collections::HashMap;

use anoncreds::{
    data_types::{
        cred_def::{CredentialDefinition, CredentialDefinitionId},
        schema::{Schema, SchemaId},
    },
    tails::TailsFileWriter,
    types::{
        Credential, CredentialDefinitionConfig, CredentialDefinitionPrivate,
        CredentialKeyCorrectnessProof, CredentialOffer, CredentialRequest,
        CredentialRequestMetadata, LinkSecret, MakeCredentialValues, PresentCredentials,
        Presentation, PresentationRequest, RegistryType, RevocationRegistryDefinition,
        RevocationRegistryDefinitionPrivate, SignatureType,
    },
};

use crate::{
    anoncreds_eth_registry::{
        address_as_did, get_cred_def, get_schema, submit_cred_def, submit_rev_reg_def,
        submit_schema, CredDefIdParts, RevRegIdParts, SchemaIdParts,
    },
    EtherSigner,
};

pub struct Holder {
    signer: EtherSigner,
    link_secret: LinkSecret,
    link_secret_id: String,
    protocol_data: HolderProtocolFlowData,
}

#[derive(Default)]
pub struct HolderProtocolFlowData {
    request_metadata: Option<CredentialRequestMetadata>,
    stored_credential: Option<Credential>,
}

impl Holder {
    pub async fn bootstrap(signer: EtherSigner) -> Self {
        let link_secret = anoncreds::prover::create_link_secret().unwrap();

        Holder {
            signer,
            link_secret,
            link_secret_id: String::from("main"),
            protocol_data: Default::default(),
        }
    }

    async fn fetch_schema(&self, schema_id: &str) -> Schema {
        // fetch schema from ledger
        println!("Holder: fetching schema...");
        get_schema(&self.signer, schema_id).await
    }

    async fn fetch_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        // fetch cred def from ledger
        println!("Holder: fetching cred def...");
        get_cred_def(&self.signer, cred_def_id).await
    }

    pub async fn accept_offer(&mut self, cred_offer: &CredentialOffer) -> CredentialRequest {
        let fetched_cred_def = self.fetch_cred_def(&cred_offer.cred_def_id.0).await;

        let (cred_request, cred_request_metadata) = anoncreds::prover::create_credential_request(
            Some(&uuid::Uuid::new_v4().to_string()),
            None,
            &fetched_cred_def,
            &self.link_secret,
            &self.link_secret_id,
            cred_offer,
        )
        .unwrap();

        self.protocol_data.request_metadata = Some(cred_request_metadata);

        cred_request
    }

    pub async fn store_credential(&mut self, mut credential: Credential) {
        let fetched_cred_def = self.fetch_cred_def(&credential.cred_def_id.0).await;

        anoncreds::prover::process_credential(
            &mut credential,
            self.protocol_data.request_metadata.as_ref().unwrap(),
            &self.link_secret,
            &fetched_cred_def,
            None,
        )
        .unwrap();

        self.protocol_data.stored_credential = Some(credential);
    }

    pub fn get_credential(&self) -> &Credential {
        self.protocol_data.stored_credential.as_ref().unwrap()
    }

    pub async fn present_credential(
        &self,
        presentation_request: &PresentationRequest,
    ) -> Presentation {
        let holder_cred = self.protocol_data.stored_credential.as_ref().unwrap();

        // construct schemas
        let mut schemas: HashMap<&SchemaId, &Schema> = HashMap::new();
        let schema_for_cred = self.fetch_schema(&holder_cred.schema_id.0).await;
        schemas.insert(&holder_cred.schema_id, &schema_for_cred);

        // construct cred defs
        let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        let cred_def_for_cred = self.fetch_cred_def(&holder_cred.cred_def_id.0).await;
        cred_defs.insert(&holder_cred.cred_def_id, &cred_def_for_cred);

        // specify creds to use for referents
        let mut creds_to_present = PresentCredentials::default();
        let mut added_cred = creds_to_present.add_credential(&holder_cred, None, None);
        added_cred.add_requested_attribute("reft1", true);

        anoncreds::prover::create_presentation(
            presentation_request,
            creds_to_present,
            None,
            &self.link_secret,
            &schemas,
            &cred_defs,
        )
        .unwrap()
    }
}

#[allow(unused)]
pub struct Issuer {
    signer: EtherSigner,
    schema_id_parts: SchemaIdParts,
    cred_def_id_parts: CredDefIdParts,
    rev_reg_id_parts: RevRegIdParts,
    schema: Schema,
    cred_def: CredentialDefinition,
    cred_def_private: CredentialDefinitionPrivate,
    correctness_proof: CredentialKeyCorrectnessProof,
    rev_reg_def: RevocationRegistryDefinition,
    rev_reg_def_private: RevocationRegistryDefinitionPrivate,
    protocol_data: IssuerProtocolFlowData,
}

/// lazily set data from protocol flows
#[derive(Default)]
pub struct IssuerProtocolFlowData {
    cred_offer: Option<CredentialOffer>,
}

impl Issuer {
    pub async fn bootstrap(signer: EtherSigner) -> Self {
        let signer_address = signer.address();
        let issuer_id = address_as_did(&signer_address);

        let attr_names: &[&str] = &["name", "age"];

        let schema_name = format!("MySchema-{}", uuid::Uuid::new_v4().to_string());
        println!("Issuer: creating schema for schema name: {schema_name}...");
        let schema = anoncreds::issuer::create_schema(
            &schema_name,
            "1.0",
            issuer_id.clone(),
            attr_names.into(),
        )
        .unwrap();

        // upload to ledger
        println!("Issuer: submitting schema...");
        let schema_id_parts = submit_schema(&signer, &schema).await;
        let schema_id = schema_id_parts.to_id();

        let cred_def_tag = format!("MyCredDef-{}", uuid::Uuid::new_v4().to_string());
        println!("Issuer: creating cred def for tag: {cred_def_tag}...");
        let (cred_def, cred_def_private, correctness_proof) =
            anoncreds::issuer::create_credential_definition(
                schema_id.clone(),
                &schema,
                issuer_id.clone(),
                &cred_def_tag,
                SignatureType::CL,
                CredentialDefinitionConfig::new(true),
            )
            .unwrap();

        // upload to ledger
        println!("Issuer: submitting cred def...");
        let cred_def_id_parts = submit_cred_def(&signer, &cred_def).await;

        let rev_reg_def_tag = format!("MyRevRegDef-{}", uuid::Uuid::new_v4().to_string());
        println!("Issuer: creating rev reg def for tag: {rev_reg_def_tag}...");

        let mut tw = TailsFileWriter::new(None);
        let (rev_reg_def, rev_reg_def_private) = anoncreds::issuer::create_revocation_registry_def(
            &cred_def,
            cred_def_id_parts.to_id(),
            issuer_id,
            &rev_reg_def_tag,
            RegistryType::CL_ACCUM,
            1000,
            &mut tw,
        )
        .unwrap();

        // upload to ledger
        println!("Issuer: submitting rev reg def...");
        let rev_reg_id_parts = submit_rev_reg_def(&signer, &rev_reg_def).await;

        let schema_id = schema_id_parts.to_id();
        let cred_def_id = cred_def_id_parts.to_id();
        let rev_reg_id = rev_reg_id_parts.to_id();
        println!(
            "Issuer: ledger data created. \n
            \tSchema ID: {schema_id}. \n
            \tCred Def ID: {cred_def_id}. \n
            \tRev Reg ID: {rev_reg_id}"
        );

        Self {
            signer,
            schema_id_parts,
            cred_def_id_parts,
            rev_reg_id_parts,
            schema,
            cred_def,
            cred_def_private,
            correctness_proof,
            rev_reg_def,
            rev_reg_def_private,
            protocol_data: Default::default(),
        }
    }

    pub fn create_offer(&mut self) -> CredentialOffer {
        let offer = anoncreds::issuer::create_credential_offer(
            self.schema_id_parts.to_id(),
            self.cred_def_id_parts.to_id(),
            &self.correctness_proof,
        )
        .unwrap();

        let offer_clone = serde_json::from_str(&serde_json::to_string(&offer).unwrap()).unwrap();
        self.protocol_data.cred_offer = Some(offer_clone);
        offer
    }

    pub fn create_credential(
        &self,
        cred_request: &CredentialRequest,
        name: impl Into<String>,
        age: impl Into<String>,
    ) -> Credential {
        let mut credential_values = MakeCredentialValues::default();
        credential_values.add_raw("name", name).unwrap();
        credential_values.add_raw("age", age).unwrap();

        anoncreds::issuer::create_credential(
            &self.cred_def,
            &self.cred_def_private,
            &self.protocol_data.cred_offer.as_ref().unwrap(),
            cred_request,
            credential_values.into(),
            // TODO - rev stuff
            None,
            None,
            None,
        )
        .unwrap()
    }
}

pub struct Verifier {
    signer: EtherSigner,
    protocol_data: VerifierProtocolFlowData,
}

#[derive(Default)]
pub struct VerifierProtocolFlowData {
    proof_request: Option<PresentationRequest>,
}

impl Verifier {
    pub fn bootstrap(signer: EtherSigner) -> Self {
        Verifier {
            signer,
            protocol_data: Default::default(),
        }
    }

    async fn fetch_schema(&self, schema_id: &str) -> Schema {
        // fetch schema from ledger
        println!("Holder: fetching schema...");
        get_schema(&self.signer, schema_id).await
    }

    async fn fetch_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        // fetch cred def from ledger
        println!("Holder: fetching cred def...");
        get_cred_def(&self.signer, cred_def_id).await
    }

    pub fn request_presentation(&mut self, from_cred_def: &str) -> PresentationRequest {
        let nonce = anoncreds::verifier::generate_nonce().unwrap();

        let proof_req_raw = serde_json::json!({
            "nonce": nonce,
            "name":"example_presentation_request",
            "version":"0.1",
            "requested_attributes":{
                "reft1":{
                    "name":"age",
                    "restrictions": {
                        "cred_def_id": from_cred_def
                    }
                },
            },
        });

        self.protocol_data.proof_request =
            Some(serde_json::from_value(proof_req_raw.clone()).unwrap());

        serde_json::from_value(proof_req_raw).unwrap()
    }

    pub async fn verify_presentation(&self, presentation: &Presentation) -> bool {
        let anoncred_resources_ids = presentation.identifiers.first().unwrap();
        let schema_id = &anoncred_resources_ids.schema_id;
        let cred_def_id = &anoncred_resources_ids.cred_def_id;

        // construct schemas
        let mut schemas: HashMap<&SchemaId, &Schema> = HashMap::new();
        let schema_for_cred = self.fetch_schema(&schema_id.0).await;
        schemas.insert(&schema_id, &schema_for_cred);

        // construct cred defs
        let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        let cred_def_for_cred = self.fetch_cred_def(&cred_def_id.0).await;
        cred_defs.insert(&cred_def_id, &cred_def_for_cred);

        anoncreds::verifier::verify_presentation(
            presentation,
            self.protocol_data.proof_request.as_ref().unwrap(),
            &schemas,
            &cred_defs,
            None,
            None,
            None,
        )
        .unwrap()
    }
}
