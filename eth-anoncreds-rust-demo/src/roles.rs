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
use chrono::{TimeZone, Utc};

use crate::{
    ledger::ledger_data::status_list_update_ledger_data::StatusListUpdateLedgerData,
    ledger::{
        contracts::{eth_did_registry::DidEthRegistry, EtherSigner},
        did_linked_resources::{
            registar::EthrDidLinkedResourcesRegistar, resolver::EthrDidLinkedResourcesResolver,
            types::input::ResourceInput,
        },
        ledger_data::LedgerDataTransformer,
    },
    utils::{random_id, serde_clone},
};

/// ID/index of the issued credential in the revocation status list
/// NOTE: there are issues with having this index at `0`, so starting
/// the index at 1 instead
const CRED_REV_ID: u32 = 1;

// https://docs.cheqd.io/identity/advanced/anoncreds/schema
const SCHEMA_RESOURCE_TYPE: &str = "anonCredsSchema";
// https://docs.cheqd.io/identity/advanced/anoncreds/credential-definition
const CRED_DEF_RESOURCE_TYPE: &str = "anonCredsCredDef";
// https://docs.cheqd.io/identity/advanced/anoncreds/revocation-registry-definition
const REV_REG_DEF_RESOURCE_TYPE: &str = "anonCredsRevocRegDef";
// https://docs.cheqd.io/identity/advanced/anoncreds/revocation-status-list
const STATUS_LIST_RESOURCE_TYPE: &str = "anonCredsStatusList";

const BINARY_MEDIA_TYPE: &str = "application/octet-stream";

pub struct Holder {
    dlr_resolver: EthrDidLinkedResourcesResolver,
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
        let resolver = EthrDidLinkedResourcesResolver::new();

        Holder {
            dlr_resolver: resolver,
            link_secret,
            link_secret_id: String::from("main"),
            protocol_data: Default::default(),
        }
    }

    async fn fetch_schema(&self, schema_id: &str) -> Schema {
        // fetch schema from ledger
        println!("Holder: fetching schema...");
        let resource = self.dlr_resolver.resolve_query(schema_id).await.unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
    }

    async fn fetch_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        // fetch cred def from ledger
        println!("Holder: fetching cred def...");
        let resource = self.dlr_resolver.resolve_query(cred_def_id).await.unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
    }

    async fn fetch_rev_reg_def(&self, rev_reg_def_id: &str) -> RevocationRegistryDefinition {
        // fetch rev reg def from ledger
        println!("Holder: fetching rev reg def...");
        let resource = self
            .dlr_resolver
            .resolve_query(rev_reg_def_id)
            .await
            .unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
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
        let fetched_rev_reg_def = self
            .fetch_rev_reg_def(&credential.rev_reg_id.as_ref().unwrap().0)
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
        let rev_reg_id = &holder_cred.rev_reg_id.as_ref().unwrap().0;
        let rev_reg_def = self.fetch_rev_reg_def(rev_reg_id).await;
        let requested_nrp_timestamp = presentation_request
            .value()
            .non_revoked
            .as_ref()
            .unwrap()
            .to
            .unwrap();
        let (rev_status_list, update_timestamp) = fetch_rev_status_list_as_of_timestamp(
            &self.dlr_resolver,
            &rev_reg_id,
            &rev_reg_def,
            requested_nrp_timestamp,
        )
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
    dlr_registar: EthrDidLinkedResourcesRegistar<EtherSigner>,
    did_registry: DidEthRegistry,
    demo_data: IssuerDemoData,
}

#[allow(unused)]
struct IssuerDemoData {
    schema_id: String,
    cred_def_id: String,
    rev_reg_def_id: String,
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
        let dlr_registar = EthrDidLinkedResourcesRegistar::new(signer.clone());

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
        let schema_resource = dlr_registar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: schema_name.to_owned(),
                    resource_type: SCHEMA_RESOURCE_TYPE.to_owned(),
                    resource_version_id: "1.0".to_owned(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(schema.clone()),
                },
            )
            .await
            .unwrap();
        let schema_id = schema_resource.resource_uri;

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
        let cred_def_resource = dlr_registar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: cred_def_tag.to_owned(),
                    resource_type: CRED_DEF_RESOURCE_TYPE.to_owned(),
                    resource_version_id: String::new(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(serde_clone(&cred_def)),
                },
            )
            .await
            .unwrap();
        let cred_def_id = cred_def_resource.resource_uri;

        let rev_reg_def_tag = "MyRevRegDef";
        println!("Issuer: creating rev reg def for tag: {rev_reg_def_tag}...");

        let mut tw = TailsFileWriter::new(Some(String::from(TAILS_DIR)));
        let (rev_reg_def, rev_reg_def_private) = anoncreds::issuer::create_revocation_registry_def(
            &cred_def,
            cred_def_id.clone(),
            issuer_did.clone(),
            rev_reg_def_tag,
            RegistryType::CL_ACCUM,
            100,
            &mut tw,
        )
        .unwrap();

        // upload to ledger
        println!("Issuer: submitting rev reg def...");
        let rev_reg_def_resource = dlr_registar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: rev_reg_def_tag.to_owned(),
                    resource_type: REV_REG_DEF_RESOURCE_TYPE.to_owned(),
                    resource_version_id: String::new(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(rev_reg_def.clone()),
                },
            )
            .await
            .unwrap();
        let rev_reg_def_id = rev_reg_def_resource.resource_uri;

        println!("Issuer: creating rev list...");
        let rev_list = anoncreds::issuer::create_revocation_status_list(
            rev_reg_def_id.clone(),
            &rev_reg_def,
            issuer_did.clone(),
            None,
            true,
        )
        .unwrap();
        println!("Issuer: submitting rev list initial entry...");
        let rev_list_resource = dlr_registar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: rev_reg_def_tag.to_owned(),
                    resource_type: STATUS_LIST_RESOURCE_TYPE.to_owned(),
                    resource_version_id: String::new(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(
                        StatusListUpdateLedgerData::from_anoncreds_data(&rev_list),
                    ),
                },
            )
            .await
            .unwrap();
        let ledger_timestamp = rev_list_resource.created.timestamp() as u64;

        println!("Issuer: submitted rev list initial entry at ledger time: {ledger_timestamp:?}");
        let rev_list = anoncreds::issuer::update_revocation_status_list_timestamp_only(
            ledger_timestamp,
            &rev_list,
        );

        println!(
            "Issuer: ledger data created. \n
            \tSchema ID: {schema_id}. \n
            \tCred Def ID: {cred_def_id}. \n
            \tRev Reg ID: {rev_reg_def_id}"
        );

        Self {
            dlr_registar,
            did_registry: DidEthRegistry,
            issuer_did,
            signer,
            demo_data: IssuerDemoData {
                schema_id,
                cred_def_id,
                rev_reg_def_id,
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

    pub async fn write_resource<T: LedgerDataTransformer>(
        &self,
        resource: T,
    ) -> Result<(), Box<dyn Error>> {
        self.dlr_registar
            .create_resource(
                &self.issuer_did,
                ResourceInput {
                    resource_name: random_id(),
                    resource_type: random_id(),
                    resource_version_id: String::new(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(resource),
                },
            )
            .await?;

        Ok(())
    }

    pub fn create_offer(&mut self) -> CredentialOffer {
        let offer = anoncreds::issuer::create_credential_offer(
            self.demo_data.schema_id.clone(),
            self.demo_data.cred_def_id.clone(),
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
                self.demo_data.rev_reg_def_id.clone(),
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

        let new_rev_list_resource = self
            .dlr_registar
            .create_resource(
                &self.issuer_did,
                ResourceInput {
                    resource_name: self.demo_data.rev_reg_def.tag.clone(),
                    resource_type: STATUS_LIST_RESOURCE_TYPE.to_owned(),
                    resource_version_id: String::new(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(
                        StatusListUpdateLedgerData::from_anoncreds_data(&new_list),
                    ),
                },
            )
            .await
            .unwrap();
        let ledger_timestamp = new_rev_list_resource.created.timestamp() as u64;

        println!("Issuer: submitted rev list update entry at ledger time: {ledger_timestamp:?}");

        let new_list = anoncreds::issuer::update_revocation_status_list_timestamp_only(
            ledger_timestamp,
            &new_list,
        );

        self.demo_data.rev_list = new_list;
    }
}

pub struct Verifier {
    dlr_resolver: EthrDidLinkedResourcesResolver,
    protocol_data: VerifierProtocolFlowData,
}

#[derive(Default)]
pub struct VerifierProtocolFlowData {
    proof_request: Option<PresentationRequest>,
}

impl Verifier {
    pub fn bootstrap() -> Self {
        let dlr_resolver = EthrDidLinkedResourcesResolver::new();

        Verifier {
            dlr_resolver,
            protocol_data: Default::default(),
        }
    }

    async fn fetch_schema(&self, schema_id: &str) -> Schema {
        // fetch schema from ledger
        println!("Verifier: fetching schema...");
        let resource = self.dlr_resolver.resolve_query(schema_id).await.unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
    }

    async fn fetch_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        // fetch cred def from ledger
        println!("Verifier: fetching cred def...");
        let resource = self.dlr_resolver.resolve_query(cred_def_id).await.unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
    }

    async fn fetch_rev_reg_def(&self, rev_reg_def_id: &str) -> RevocationRegistryDefinition {
        // fetch rev reg def from ledger
        println!("Verifier: fetching rev reg def...");
        let resource = self
            .dlr_resolver
            .resolve_query(rev_reg_def_id)
            .await
            .unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
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

        // construct rev reg def
        let rev_reg_def_for_cred = self.fetch_rev_reg_def(&rev_reg_id).await;

        // construct rev info
        let (rev_status_list, _update_timestamp) = fetch_rev_status_list_as_of_timestamp(
            &self.dlr_resolver,
            &rev_reg_id,
            &rev_reg_def_for_cred,
            presented_timestamp,
        )
        .await;

        let mut rev_reg_defs: HashMap<
            &RevocationRegistryDefinitionId,
            &RevocationRegistryDefinition,
        > = HashMap::new();

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

async fn fetch_rev_status_list_as_of_timestamp(
    dlr_resolver: &EthrDidLinkedResourcesResolver,
    rev_reg_id: &str,
    rev_reg_def: &RevocationRegistryDefinition,
    timestamp: u64,
) -> (RevocationStatusList, u64) {
    let issuer_did = &rev_reg_def.issuer_id.0;
    let rev_reg_name = &rev_reg_def.tag;
    let version_time = Utc.timestamp_opt(timestamp as i64, 0).unwrap().to_rfc3339(); // TODO - does this have Z?
    let version_time_url = urlencoding::encode(&version_time);

    // https://docs.cheqd.io/identity/advanced/anoncreds/revocation-status-list#obtain-status-list-content-at-a-point-in-time
    // did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J?universityDegree&resourceType=anonCredsStatusList&versionTime=2022-08-21T08:40:00Z
    // NOTE ^ i think above is missing resourceName=universityDegree
    let query = format!("{issuer_did}?resourceName={rev_reg_name}&resourceType={STATUS_LIST_RESOURCE_TYPE}&versionTime={version_time_url}");
    let resource = dlr_resolver.resolve_query(&query).await.unwrap();

    let resource_timestamp = resource.created.timestamp() as u64;
    let rev_list_ledger_data: StatusListUpdateLedgerData =
        LedgerDataTransformer::from_ledger_bytes(&resource.content);

    let rev_list = rev_list_ledger_data.into_anoncreds_data(resource_timestamp, rev_reg_id);

    (rev_list, resource_timestamp)
}
