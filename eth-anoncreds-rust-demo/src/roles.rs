use std::{
    collections::{BTreeSet, HashMap},
    error::Error,
    sync::Arc,
};

use anoncreds::{
    data_types::{
        cred_def::{CredentialDefinition, CredentialDefinitionId},
        rev_reg::RevocationRegistryId,
        rev_reg_def::RevocationRegistryDefinitionId,
        schema::{Schema, SchemaId},
    },
    prover::create_or_update_revocation_state,
    tails::{TailsFileReader, TailsFileWriter},
    types::{
        Credential, CredentialDefinitionConfig, CredentialDefinitionPrivate,
        CredentialKeyCorrectnessProof, CredentialOffer, CredentialRequest,
        CredentialRequestMetadata, CredentialRevocationConfig, LinkSecret, MakeCredentialValues,
        PresentCredentials, Presentation, PresentationRequest, RegistryType,
        RevocationRegistryDefinition, RevocationRegistryDefinitionPrivate, RevocationStatusList,
        SignatureType,
    },
};
use serde::Serialize;

use crate::{
    anoncreds_eth_registry::{AnoncredsEthRegistry, DIDResourceId, EtherSigner},
    eth_did_registry::DidEthRegistry,
};

/// ID/index of the issued credential in the revocation status list
/// NOTE: there are issues with having this index at `0`, so starting
/// the index at 1 instead
const CRED_REV_ID: u32 = 1;

pub struct Holder {
    registry: AnoncredsEthRegistry,
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
    pub async fn bootstrap() -> Self {
        let link_secret = anoncreds::prover::create_link_secret().unwrap();
        let registry = AnoncredsEthRegistry;

        Holder {
            registry,
            link_secret,
            link_secret_id: String::from("main"),
            protocol_data: Default::default(),
        }
    }

    async fn fetch_schema(&self, schema_id: &str) -> Schema {
        // fetch schema from ledger
        println!("Holder: fetching schema...");
        self.registry.get_json_resource(schema_id).await
    }

    async fn fetch_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        // fetch cred def from ledger
        println!("Holder: fetching cred def...");
        self.registry.get_json_resource(cred_def_id).await
    }

    pub async fn accept_offer(&mut self, cred_offer: &CredentialOffer) -> CredentialRequest {
        let fetched_cred_def = self.fetch_cred_def(&cred_offer.cred_def_id.0).await;

        let (cred_request, cred_request_metadata) = anoncreds::prover::create_credential_request(
            Some("entropy"),
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
        let fetched_rev_reg_def: RevocationRegistryDefinition = self
            .registry
            .get_json_resource(&credential.rev_reg_id.as_ref().unwrap().0)
            .await;

        anoncreds::prover::process_credential(
            &mut credential,
            self.protocol_data.request_metadata.as_ref().unwrap(),
            &self.link_secret,
            &fetched_cred_def,
            Some(&fetched_rev_reg_def),
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

    pub async fn present_credential_with_nrp(
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

        // construct rev_state
        let rev_reg_def: RevocationRegistryDefinition = self
            .registry
            .get_json_resource(&holder_cred.rev_reg_id.as_ref().unwrap().0)
            .await;
        let requested_nrp_timestamp = presentation_request
            .value()
            .non_revoked
            .as_ref()
            .unwrap()
            .to
            .unwrap();
        let rev_reg_id = &holder_cred.rev_reg_id.as_ref().unwrap().0;
        let (rev_status_list, update_timestamp) = self
            .registry
            .get_rev_reg_status_list_as_of_timestamp(rev_reg_id, requested_nrp_timestamp)
            .await;

        let rev_reg_idx = holder_cred.signature.extract_index().unwrap();
        let rev_state = create_or_update_revocation_state(
            &rev_reg_def.value.tails_location,
            &rev_reg_def,
            &rev_status_list,
            rev_reg_idx,
            None,
            None,
        )
        .unwrap();

        // specify creds to use for referents
        let mut creds_to_present = PresentCredentials::default();
        let mut added_cred =
            creds_to_present.add_credential(&holder_cred, Some(update_timestamp), Some(&rev_state));
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

const TAILS_DIR: &str = "tails";

pub struct Issuer {
    pub issuer_did: String,
    pub signer: Arc<EtherSigner>,
    anoncreds_registry: AnoncredsEthRegistry,
    did_registry: DidEthRegistry,
    demo_data: IssuerDemoData,
}

#[allow(unused)]
struct IssuerDemoData {
    schema_resource_id: DIDResourceId,
    cred_def_resource_id: DIDResourceId,
    rev_reg_def_resource_id: DIDResourceId,
    schema: Schema,
    cred_def: CredentialDefinition,
    cred_def_private: CredentialDefinitionPrivate,
    correctness_proof: CredentialKeyCorrectnessProof,
    rev_reg_def: RevocationRegistryDefinition,
    rev_reg_def_private: RevocationRegistryDefinitionPrivate,
    rev_list: RevocationStatusList,
    protocol_data: IssuerProtocolFlowData,
}

/// lazily set data from protocol flows
#[derive(Default)]
pub struct IssuerProtocolFlowData {
    cred_offer: Option<CredentialOffer>,
}

#[derive(Debug)]
pub enum CredRevocationUpdateType {
    Revoke,
    Issue,
}

impl Issuer {
    pub async fn bootstrap(issuer_did: String, signer: Arc<EtherSigner>) -> Self {
        let anoncreds_registry = AnoncredsEthRegistry;

        let attr_names: &[&str] = &["name", "age"];

        let schema_name = "MySchema";
        println!("Issuer: creating schema for schema name: {schema_name}...");
        let schema = anoncreds::issuer::create_schema(
            schema_name,
            "1.0",
            issuer_did.clone(),
            attr_names.into(),
        )
        .unwrap();

        // upload to ledger
        println!("Issuer: submitting schema...");
        let schema_resource_id = anoncreds_registry
            .submit_json_resource(signer.clone(), &issuer_did, &schema, "schema")
            .await
            .unwrap();
        let schema_id = schema_resource_id.to_id();

        let cred_def_tag = "MyCredDef";
        println!("Issuer: creating cred def for tag: {cred_def_tag}...");
        let (cred_def, cred_def_private, correctness_proof) =
            anoncreds::issuer::create_credential_definition(
                schema_id.clone(),
                &schema,
                issuer_did.clone(),
                cred_def_tag,
                SignatureType::CL,
                CredentialDefinitionConfig::new(true),
            )
            .unwrap();

        // upload to ledger
        println!("Issuer: submitting cred def...");
        let cred_def_resource_id = anoncreds_registry
            .submit_json_resource(signer.clone(), &issuer_did, &cred_def, "cred_def")
            .await
            .unwrap();

        let rev_reg_def_tag = "MyRevRegDef";
        println!("Issuer: creating rev reg def for tag: {rev_reg_def_tag}...");

        let mut tw = TailsFileWriter::new(Some(String::from(TAILS_DIR)));
        let (rev_reg_def, rev_reg_def_private) = anoncreds::issuer::create_revocation_registry_def(
            &cred_def,
            cred_def_resource_id.to_id(),
            issuer_did.clone(),
            rev_reg_def_tag,
            RegistryType::CL_ACCUM,
            100,
            &mut tw,
        )
        .unwrap();

        // upload to ledger
        println!("Issuer: submitting rev reg def...");
        let rev_reg_def_resource_id = anoncreds_registry
            .submit_json_resource(signer.clone(), &issuer_did, &rev_reg_def, "rev_reg_def")
            .await
            .unwrap();

        println!("Issuer: creating rev list...");
        let rev_list = anoncreds::issuer::create_revocation_status_list(
            rev_reg_def_resource_id.to_id(),
            &rev_reg_def,
            issuer_did.clone(),
            None,
            true,
        )
        .unwrap();
        println!("Issuer: submitting rev list initial entry...");
        let ledger_timestamp = anoncreds_registry
            .submit_rev_reg_status_list_update(
                signer.clone(),
                &issuer_did,
                &rev_reg_def_resource_id.to_id(),
                &rev_list,
            )
            .await;
        println!("Issuer: submitted rev list initial entry at ledger time: {ledger_timestamp:?}");
        let rev_list = anoncreds::issuer::update_revocation_status_list_timestamp_only(
            ledger_timestamp,
            &rev_list,
        );

        let schema_id = schema_resource_id.to_id();
        let cred_def_id = cred_def_resource_id.to_id();
        let rev_reg_id = rev_reg_def_resource_id.to_id();
        println!(
            "Issuer: ledger data created. \n
            \tSchema ID: {schema_id}. \n
            \tCred Def ID: {cred_def_id}. \n
            \tRev Reg ID: {rev_reg_id}"
        );

        Self {
            anoncreds_registry,
            did_registry: DidEthRegistry,
            issuer_did,
            signer,
            demo_data: IssuerDemoData {
                schema_resource_id,
                cred_def_resource_id,
                rev_reg_def_resource_id,
                schema,
                cred_def,
                cred_def_private,
                correctness_proof,
                rev_reg_def,
                rev_reg_def_private,
                rev_list,
                protocol_data: Default::default(),
            },
        }
    }

    pub fn change_signer(&mut self, new_signer: Arc<EtherSigner>) {
        self.signer = new_signer;
    }

    pub async fn rotate_did_controller(&self, new_controller: &Arc<EtherSigner>) {
        let signer = self.signer.clone();

        self.did_registry
            .change_owner(signer, &self.issuer_did, new_controller.address())
            .await;
    }

    pub async fn write_resource<T: Serialize>(&self, resource: &T) -> Result<(), Box<dyn Error>> {
        let signer = self.signer.clone();
        self.anoncreds_registry
            .submit_json_resource(signer, &self.issuer_did, resource, "misc")
            .await?;
        Ok(())
    }

    pub fn create_offer(&mut self) -> CredentialOffer {
        let offer = anoncreds::issuer::create_credential_offer(
            self.demo_data.schema_resource_id.to_id(),
            self.demo_data.cred_def_resource_id.to_id(),
            &self.demo_data.correctness_proof,
        )
        .unwrap();

        let offer_clone = serde_json::from_str(&serde_json::to_string(&offer).unwrap()).unwrap();
        self.demo_data.protocol_data.cred_offer = Some(offer_clone);
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

        let tr =
            TailsFileReader::new_tails_reader(&self.demo_data.rev_reg_def.value.tails_location);

        anoncreds::issuer::create_credential(
            &self.demo_data.cred_def,
            &self.demo_data.cred_def_private,
            &self.demo_data.protocol_data.cred_offer.as_ref().unwrap(),
            cred_request,
            credential_values.into(),
            Some(RevocationRegistryId::new_unchecked(
                self.demo_data.rev_reg_def_resource_id.to_id(),
            )),
            Some(&self.demo_data.rev_list),
            Some(CredentialRevocationConfig {
                reg_def: &self.demo_data.rev_reg_def,
                reg_def_private: &self.demo_data.rev_reg_def_private,
                registry_idx: CRED_REV_ID,
                tails_reader: tr,
            }),
        )
        .unwrap()
    }

    pub async fn update_credential_revocation(&mut self, update_type: CredRevocationUpdateType) {
        // NOTE - these lists seem to be the delta (i.e. changes to be made) rather than complete list
        let mut update_list: BTreeSet<u32> = BTreeSet::new();
        update_list.insert(CRED_REV_ID);

        // if requested update is to 'revoke', then set `revoked_updates` to Some, else `issued_updates`
        // as Some.
        let (issued_updates, revoked_updates) = match update_type {
            CredRevocationUpdateType::Issue => (Some(update_list), None),
            CredRevocationUpdateType::Revoke => (None, Some(update_list)),
        };

        println!("Issuer: submitting rev list update entry for update type: {update_type:?}");

        let new_list = anoncreds::issuer::update_revocation_status_list(
            None,
            issued_updates,
            revoked_updates,
            &self.demo_data.rev_reg_def,
            &self.demo_data.rev_list,
        )
        .unwrap();

        let ledger_timestamp = self
            .anoncreds_registry
            .submit_rev_reg_status_list_update(
                self.signer.clone(),
                &self.issuer_did,
                &self.demo_data.rev_reg_def_resource_id.to_id(),
                &new_list,
            )
            .await;

        println!("Issuer: submitted rev list update entry at ledger time: {ledger_timestamp:?}");

        let new_list = anoncreds::issuer::update_revocation_status_list_timestamp_only(
            ledger_timestamp,
            &new_list,
        );

        self.demo_data.rev_list = new_list;
    }
}

pub struct Verifier {
    registry: AnoncredsEthRegistry,
    protocol_data: VerifierProtocolFlowData,
}

#[derive(Default)]
pub struct VerifierProtocolFlowData {
    proof_request: Option<PresentationRequest>,
}

impl Verifier {
    pub fn bootstrap() -> Self {
        let registry = AnoncredsEthRegistry;

        Verifier {
            registry,
            protocol_data: Default::default(),
        }
    }

    async fn fetch_schema(&self, schema_id: &str) -> Schema {
        // fetch schema from ledger
        println!("Holder: fetching schema...");
        self.registry.get_json_resource(schema_id).await
    }

    async fn fetch_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        // fetch cred def from ledger
        println!("Holder: fetching cred def...");
        self.registry.get_json_resource(cred_def_id).await
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

    pub fn request_presentation_with_nrp(
        &mut self,
        from_cred_def: &str,
        non_revoked_as_of: u64,
    ) -> PresentationRequest {
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
                    },
                },
            },
            "non_revoked": {
                "to": non_revoked_as_of
            }
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

    pub async fn verify_presentation_with_nrp(&self, presentation: &Presentation) -> bool {
        let anoncred_resources_ids = presentation.identifiers.first().unwrap();
        let schema_id = &anoncred_resources_ids.schema_id;
        let cred_def_id = &anoncred_resources_ids.cred_def_id;
        let rev_reg_id = anoncred_resources_ids.rev_reg_id.clone().unwrap().0;
        let presented_timestamp = anoncred_resources_ids.timestamp.unwrap();

        // construct schemas
        let mut schemas: HashMap<&SchemaId, &Schema> = HashMap::new();
        let schema_for_cred = self.fetch_schema(&schema_id.0).await;
        schemas.insert(&schema_id, &schema_for_cred);

        // construct cred defs
        let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        let cred_def_for_cred = self.fetch_cred_def(&cred_def_id.0).await;
        cred_defs.insert(&cred_def_id, &cred_def_for_cred);

        // construct rev info
        let rev_status_list = self
            .registry
            .get_rev_reg_status_list_as_of_timestamp(&rev_reg_id, presented_timestamp)
            .await
            .0;
        let mut rev_reg_defs: HashMap<
            &RevocationRegistryDefinitionId,
            &RevocationRegistryDefinition,
        > = HashMap::new();
        let rev_reg_def_for_cred = self.registry.get_json_resource(&rev_reg_id).await;
        // re-typing from RevocationRegistryId to RevocationRegistryDefinitionId?! seems to be the same thing?
        let rev_reg_def_id = rev_reg_id.try_into().unwrap();
        rev_reg_defs.insert(&rev_reg_def_id, &rev_reg_def_for_cred);

        anoncreds::verifier::verify_presentation(
            presentation,
            self.protocol_data.proof_request.as_ref().unwrap(),
            &schemas,
            &cred_defs,
            Some(&rev_reg_defs),
            Some(vec![&rev_status_list]),
            None,
        )
        .unwrap()
    }
}
