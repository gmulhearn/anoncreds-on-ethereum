use std::{collections::BTreeSet, error::Error, sync::Arc};

use anoncreds::{
    data_types::{cred_def::CredentialDefinition, rev_reg::RevocationRegistryId, schema::Schema},
    tails::{TailsFileReader, TailsFileWriter},
    types::{
        Credential, CredentialDefinitionConfig, CredentialDefinitionPrivate,
        CredentialKeyCorrectnessProof, CredentialOffer, CredentialRequest,
        CredentialRevocationConfig, MakeCredentialValues, RegistryType,
        RevocationRegistryDefinition, RevocationRegistryDefinitionPrivate, RevocationStatusList,
        SignatureType,
    },
};
use did_ethr_linked_resources::{
    contracts::eth_did_registry::DidEthRegistry, registrar::EthrDidLinkedResourcesRegistrar,
    types::input::ResourceInput,
};

use crate::{
    config::DemoConfig,
    ethers_client::EtherSigner,
    utils::{random_id, serde_clone},
};
use did_ethr_anoncreds::{
    ledger_data_transformer::LedgerDataTransformer, registrar::EthrDidAnoncredsRegistrar,
};

const TAILS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tails");

/// ID/index of the issued credential in the revocation status list
/// NOTE: there are issues with having this index at `0`, so starting
/// the index at 1 instead
const CRED_REV_ID: u32 = 1;

pub struct Issuer {
    pub issuer_did: String,
    pub signer: Arc<EtherSigner>,
    anoncreds_registrar: EthrDidAnoncredsRegistrar<EtherSigner>,
    dlr_registrar: EthrDidLinkedResourcesRegistrar<EtherSigner>,
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
    pub async fn bootstrap(
        issuer_did: String,
        signer: Arc<EtherSigner>,
        conf: &DemoConfig,
    ) -> Self {
        let dlr_registrar =
            EthrDidLinkedResourcesRegistrar::new(signer.clone(), conf.get_dlr_network_config());
        let anoncreds_registrar =
            EthrDidAnoncredsRegistrar::new(signer.clone(), conf.get_dlr_network_config());

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
        let schema_resource = anoncreds_registrar
            .write_schema(&issuer_did, schema.clone())
            .await;
        let schema_id = schema_resource.metadata.resource_uri;

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
        let cred_def_resource = anoncreds_registrar
            .write_cred_def(&issuer_did, serde_clone(&cred_def))
            .await;
        let cred_def_id = cred_def_resource.metadata.resource_uri;

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
        let rev_reg_def_resource = anoncreds_registrar
            .write_rev_reg_def(&issuer_did, rev_reg_def.clone())
            .await;
        let rev_reg_def_id = rev_reg_def_resource.metadata.resource_uri;

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
        let rev_list_resource = anoncreds_registrar
            .write_rev_status_list(&issuer_did, rev_reg_def_tag, &rev_list)
            .await;
        let ledger_timestamp = rev_list_resource.metadata.created.timestamp() as u64;

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
            anoncreds_registrar,
            dlr_registrar,
            did_registry: DidEthRegistry::new(conf.get_did_ethr_network_config()),
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
        self.signer = new_signer.clone();
        self.anoncreds_registrar.change_signer(new_signer.clone());
        self.dlr_registrar.change_signer(new_signer);
    }

    pub async fn rotate_did_controller(&self, new_controller: &Arc<EtherSigner>) {
        let signer = self.signer.clone();

        self.did_registry
            .change_owner(signer, &self.issuer_did, new_controller.address())
            .await;
    }

    pub async fn write_arbitrary_resource<T: LedgerDataTransformer>(
        &self,
        resource: T,
    ) -> Result<(), Box<dyn Error>> {
        self.dlr_registrar
            .create_resource(
                &self.issuer_did,
                ResourceInput {
                    resource_name: random_id(),
                    resource_type: random_id(),
                    resource_version_id: String::new(),
                    media_type: random_id(),
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
            .anoncreds_registrar
            .write_rev_status_list(&self.issuer_did, &self.demo_data.rev_reg_def.tag, &new_list)
            .await;
        let ledger_timestamp = new_rev_list_resource.metadata.created.timestamp() as u64;

        println!("Issuer: submitted rev list update entry at ledger time: {ledger_timestamp:?}");

        let new_list = anoncreds::issuer::update_revocation_status_list_timestamp_only(
            ledger_timestamp,
            &new_list,
        );

        self.demo_data.rev_list = new_list;
    }
}
