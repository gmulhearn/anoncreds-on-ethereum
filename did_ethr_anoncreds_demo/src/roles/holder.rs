use std::collections::HashMap;

use anoncreds::{
    data_types::{
        cred_def::{CredentialDefinition, CredentialDefinitionId},
        schema::{Schema, SchemaId},
    },
    prover::create_or_update_revocation_state,
    types::{
        Credential, CredentialOffer, CredentialRequest, CredentialRequestMetadata, LinkSecret,
        PresentCredentials, Presentation, PresentationRequest,
    },
};

use did_ethr_anoncreds::resolver::EthrDidAnoncredsResolver;

pub struct Holder {
    anoncreds_resolver: EthrDidAnoncredsResolver,
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
        let resolver = EthrDidAnoncredsResolver::new();

        Holder {
            anoncreds_resolver: resolver,
            link_secret,
            link_secret_id: String::from("main"),
            protocol_data: Default::default(),
        }
    }

    pub async fn accept_offer(&mut self, cred_offer: &CredentialOffer) -> CredentialRequest {
        let fetched_cred_def = self
            .anoncreds_resolver
            .fetch_cred_def(&cred_offer.cred_def_id.0)
            .await;

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
        let fetched_cred_def = self
            .anoncreds_resolver
            .fetch_cred_def(&credential.cred_def_id.0)
            .await;
        let fetched_rev_reg_def = self
            .anoncreds_resolver
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
        let schema_for_cred = self
            .anoncreds_resolver
            .fetch_schema(&holder_cred.schema_id.0)
            .await;
        schemas.insert(&holder_cred.schema_id, &schema_for_cred);

        // construct cred defs
        let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        let cred_def_for_cred = self
            .anoncreds_resolver
            .fetch_cred_def(&holder_cred.cred_def_id.0)
            .await;
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
        let schema_for_cred = self
            .anoncreds_resolver
            .fetch_schema(&holder_cred.schema_id.0)
            .await;
        schemas.insert(&holder_cred.schema_id, &schema_for_cred);

        // construct cred defs
        let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        let cred_def_for_cred = self
            .anoncreds_resolver
            .fetch_cred_def(&holder_cred.cred_def_id.0)
            .await;
        cred_defs.insert(&holder_cred.cred_def_id, &cred_def_for_cred);

        // construct rev_state
        let rev_reg_id = &holder_cred.rev_reg_id.as_ref().unwrap().0;
        let rev_reg_def = self.anoncreds_resolver.fetch_rev_reg_def(rev_reg_id).await;
        let requested_nrp_timestamp = presentation_request
            .value()
            .non_revoked
            .as_ref()
            .unwrap()
            .to
            .unwrap();
        let (rev_status_list, update_timestamp) = self
            .anoncreds_resolver
            .fetch_rev_status_list_as_of_timestamp(
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
