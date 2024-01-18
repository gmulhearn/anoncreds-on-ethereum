use std::sync::Arc;

use anoncreds::{
    data_types::{cred_def::CredentialDefinition, schema::Schema},
    types::{RevocationRegistryDefinition, RevocationStatusList},
};
use did_ethr_linked_resources::{
    config::ContractNetworkConfig,
    registrar::EthrDidLinkedResourcesRegistrar,
    types::{input::ResourceInput, output::Resource},
};
use ethers::providers::Middleware;

use super::ledger_data_transformer::{
    status_list_update_ledger_data::StatusListUpdateLedgerData, LedgerDataTransformer,
    BINARY_MEDIA_TYPE, CRED_DEF_RESOURCE_TYPE, NO_VERSION, REV_REG_DEF_RESOURCE_TYPE,
    SCHEMA_RESOURCE_TYPE, STATUS_LIST_RESOURCE_TYPE,
};

pub struct EthrDidAnoncredsRegistrar<S> {
    dlr_registrar: EthrDidLinkedResourcesRegistrar<S>,
}

impl<S> EthrDidAnoncredsRegistrar<S>
where
    S: Middleware,
{
    pub fn new(signer: Arc<S>, dlr_config: ContractNetworkConfig) -> Self {
        Self {
            dlr_registrar: EthrDidLinkedResourcesRegistrar::new(signer, dlr_config),
        }
    }

    pub async fn write_schema(&self, issuer_did: &str, schema: Schema) -> Resource {
        self.dlr_registrar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: schema.name.clone(),
                    resource_type: SCHEMA_RESOURCE_TYPE.to_owned(),
                    resource_version_id: schema.version.clone(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(schema),
                },
            )
            .await
            .unwrap()
    }

    pub async fn write_cred_def(
        &self,
        issuer_did: &str,
        cred_def: CredentialDefinition,
    ) -> Resource {
        self.dlr_registrar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: cred_def.tag.clone(),
                    resource_type: CRED_DEF_RESOURCE_TYPE.to_owned(),
                    resource_version_id: NO_VERSION.to_owned(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(cred_def),
                },
            )
            .await
            .unwrap()
    }

    pub async fn write_rev_reg_def(
        &self,
        issuer_did: &str,
        rev_reg_def: RevocationRegistryDefinition,
    ) -> Resource {
        self.dlr_registrar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: rev_reg_def.tag.clone(),
                    resource_type: REV_REG_DEF_RESOURCE_TYPE.to_owned(),
                    resource_version_id: NO_VERSION.to_owned(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(rev_reg_def),
                },
            )
            .await
            .unwrap()
    }

    pub async fn write_rev_status_list(
        &self,
        issuer_did: &str,
        rev_reg_tag: &str,
        rev_list: &RevocationStatusList,
    ) -> Resource {
        self.dlr_registrar
            .create_resource(
                &issuer_did,
                ResourceInput {
                    resource_name: rev_reg_tag.to_owned(),
                    resource_type: STATUS_LIST_RESOURCE_TYPE.to_owned(),
                    resource_version_id: String::new(),
                    media_type: BINARY_MEDIA_TYPE.to_owned(),
                    content: LedgerDataTransformer::into_ledger_bytes(
                        StatusListUpdateLedgerData::from_anoncreds_data(&rev_list),
                    ),
                },
            )
            .await
            .unwrap()
    }
}
