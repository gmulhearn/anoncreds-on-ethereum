use std::error::Error;

use chrono::Utc;

use crate::{
    contracts::ethr_dlr_registry::{
        DLRRegistry, NewResourceFilter, ResourceVersionMetadataChainNode,
    },
    types::query::ResourceQuery,
    utils::did_identity_as_full_did,
};

use super::types::output::Resource;

pub struct EthrDidLinkedResourcesResolver {
    registry: DLRRegistry,
}

impl EthrDidLinkedResourcesResolver {
    pub fn new() -> Self {
        Self {
            registry: DLRRegistry,
        }
    }

    /// TODO
    ///
    /// Resolve an exact resource with a DLR query
    pub async fn resolve_query(&self, query: &str) -> Result<Resource, Box<dyn Error>> {
        let query = ResourceQuery::parse_from_str(query)?;
        let did_id = query.did_identity;
        let did = did_identity_as_full_did(&did_id);
        let params = query.parameters;

        if let Some(resource_id) = params.resource_id {
            let resource = self.registry.get_resource_by_id(&did, &resource_id).await;
            let Some(resource) = resource else {
                return Err("Not Found".into());
            };
            let metadata_node = self.resolve_metadata_chain_node_for_event(&resource).await;
            return Ok(Resource::from((resource, metadata_node)));
        }

        if params.all_resource_versions.is_some()
            || params.latest_resource_version.is_some()
            || params.linked_resource.is_some()
            || params.resource_metadata.is_some()
            || params.resource_version_id.is_some()
        {
            // probably can't support indexing on these params (without thegraph or scanning)
            return Err("Unsupported param".into());
        }

        let version_time = params.version_time.unwrap_or_else(|| Utc::now());

        let (Some(resource_name), Some(resource_type)) =
            (params.resource_name, params.resource_type)
        else {
            // other queries are not supported for now..
            return Err("Not found - too vague".into());
        };

        // resolve as a resource (known by name+type) at an epoch
        let (resource, metadata_node) = self
            .registry
            .get_resource_by_name_and_type_at_epoch(
                &did,
                &resource_name,
                &resource_type,
                version_time.timestamp() as u64,
            )
            .await
            .ok_or("Not found")?;

        Ok(Resource::from((resource, metadata_node)))
    }

    pub(super) async fn resolve_metadata_chain_node_for_event(
        &self,
        event: &NewResourceFilter,
    ) -> ResourceVersionMetadataChainNode {
        self.registry
            .get_resource_metadata_chain_node(
                &did_identity_as_full_did(&event.did_identity),
                &event.resource.metadata.resource_name,
                &event.resource.metadata.resource_type,
                event.resource.metadata.metadata_chain_node_index.as_u64(),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        contracts::{ethr_dlr_registry::DLRRegistry, test_utils::get_writer_ethers_client},
        types::input::ResourceInput,
        utils::did_identity_as_full_did,
    };

    #[tokio::test]
    async fn test_resolve_exact_uri() {
        let resolver = super::EthrDidLinkedResourcesResolver::new();
        let resource_name = &format!("foo{}", uuid::Uuid::new_v4());
        let resource_type = "bar";

        // create resource
        let signer = get_writer_ethers_client(0);
        let did = did_identity_as_full_did(&signer.address());

        let registry = DLRRegistry;

        let created_resource = registry
            .create_or_update_resource(
                signer.clone(),
                &did,
                ResourceInput {
                    resource_name: resource_name.to_owned(),
                    resource_type: resource_type.to_owned(),
                    resource_version_id: String::new(),
                    media_type: String::from("text/plain"),
                    content: "hello world".as_bytes().to_vec(),
                },
            )
            .await;
        dbg!(&created_resource);

        // resolve exact
        let resource_uri = format!(
            "{did}/resources/{resource_id}",
            resource_id = created_resource.resource_id
        );

        let resolved_res = resolver.resolve_query(&resource_uri).await.unwrap();
        dbg!(resolved_res);

        // resolve exact in query
        let resource_query = format!(
            "{did}?resourceId={resource_id}",
            resource_id = created_resource.resource_id
        );

        let resolved_res = resolver.resolve_query(&resource_query).await.unwrap();
        dbg!(resolved_res);
    }
}
