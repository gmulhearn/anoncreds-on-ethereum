use anoncreds::{
    data_types::{cred_def::CredentialDefinition, schema::Schema},
    types::{RevocationRegistryDefinition, RevocationStatusList},
};
use chrono::{TimeZone, Utc};
use did_ethr_linked_resources::{
    config::ContractNetworkConfig, resolver::EthrDidLinkedResourcesResolver,
};

use crate::ledger_data_transformer::LedgerDataTransformer;

use super::ledger_data_transformer::{
    status_list_update_ledger_data::StatusListUpdateLedgerData, STATUS_LIST_RESOURCE_TYPE,
};

pub struct EthrDidAnoncredsResolver {
    dlr_resolver: EthrDidLinkedResourcesResolver,
}

impl EthrDidAnoncredsResolver {
    pub fn new(dlr_config: ContractNetworkConfig) -> Self {
        Self {
            dlr_resolver: EthrDidLinkedResourcesResolver::new(dlr_config),
        }
    }

    pub async fn fetch_schema(&self, schema_id: &str) -> Schema {
        // fetch schema from ledger
        println!("Fetching schema {schema_id}...");
        let resource = self.dlr_resolver.resolve_query(schema_id).await.unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
    }

    pub async fn fetch_cred_def(&self, cred_def_id: &str) -> CredentialDefinition {
        // fetch cred def from ledger
        println!("Fetching cred def {cred_def_id}...");
        let resource = self.dlr_resolver.resolve_query(cred_def_id).await.unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
    }

    pub async fn fetch_rev_reg_def(&self, rev_reg_def_id: &str) -> RevocationRegistryDefinition {
        // fetch rev reg def from ledger
        println!("Fetching rev reg def {rev_reg_def_id}...");
        let resource = self
            .dlr_resolver
            .resolve_query(rev_reg_def_id)
            .await
            .unwrap();
        LedgerDataTransformer::from_ledger_bytes(&resource.content)
    }

    pub async fn fetch_rev_status_list_as_of_timestamp(
        &self,
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
        println!("Fetching status list with query: {query}");
        let resource = self.dlr_resolver.resolve_query(&query).await.unwrap();

        let resource_timestamp = resource.metadata.created.timestamp() as u64;
        let rev_list_ledger_data: StatusListUpdateLedgerData =
            LedgerDataTransformer::from_ledger_bytes(&resource.content);

        let rev_list = rev_list_ledger_data.into_anoncreds_data(resource_timestamp, rev_reg_id);

        (rev_list, resource_timestamp)
    }
}
