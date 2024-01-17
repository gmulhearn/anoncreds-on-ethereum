use std::collections::HashMap;

use anoncreds::{
    data_types::{
        cred_def::{CredentialDefinition, CredentialDefinitionId},
        rev_reg_def::RevocationRegistryDefinitionId,
        schema::{Schema, SchemaId},
    },
    types::{Presentation, PresentationRequest, RevocationRegistryDefinition},
};

use did_ethr_anoncreds::resolver::EthrDidAnoncredsResolver;

pub struct Verifier {
    anoncreds_resolver: EthrDidAnoncredsResolver,
    protocol_data: VerifierProtocolFlowData,
}

#[derive(Default)]
pub struct VerifierProtocolFlowData {
    proof_request: Option<PresentationRequest>,
}

impl Verifier {
    pub fn bootstrap() -> Self {
        let resolver = EthrDidAnoncredsResolver::new();

        Verifier {
            anoncreds_resolver: resolver,
            protocol_data: Default::default(),
        }
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
        let schema_for_cred = self.anoncreds_resolver.fetch_schema(&schema_id.0).await;
        schemas.insert(&schema_id, &schema_for_cred);

        // construct cred defs
        let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        let cred_def_for_cred = self.anoncreds_resolver.fetch_cred_def(&cred_def_id.0).await;
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
        let schema_for_cred = self.anoncreds_resolver.fetch_schema(&schema_id.0).await;
        schemas.insert(&schema_id, &schema_for_cred);

        // construct cred defs
        let mut cred_defs: HashMap<&CredentialDefinitionId, &CredentialDefinition> = HashMap::new();
        let cred_def_for_cred = self.anoncreds_resolver.fetch_cred_def(&cred_def_id.0).await;
        cred_defs.insert(&cred_def_id, &cred_def_for_cred);

        // construct rev reg def
        let rev_reg_def_for_cred = self.anoncreds_resolver.fetch_rev_reg_def(&rev_reg_id).await;

        // construct rev info
        let (rev_status_list, _update_timestamp) = self
            .anoncreds_resolver
            .fetch_rev_status_list_as_of_timestamp(
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
