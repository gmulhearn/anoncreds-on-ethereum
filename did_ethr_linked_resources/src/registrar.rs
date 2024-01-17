use std::{error::Error, sync::Arc};

use ethers::providers::Middleware;

use crate::{
    contracts::ethr_dlr_registry::EthrDIDLinkedResourcesRegistry, types::output::Resource,
};

use super::{resolver::EthrDidLinkedResourcesResolver, types::input::ResourceInput};

pub struct EthrDidLinkedResourcesRegistrar<S> {
    registry: EthrDIDLinkedResourcesRegistry,
    resolver: EthrDidLinkedResourcesResolver, // eh - only need this for the metadata node convenience method
    signer: Arc<S>,
}

impl<S> EthrDidLinkedResourcesRegistrar<S>
where
    S: Middleware,
{
    pub fn new(signer: Arc<S>) -> Self {
        Self {
            registry: EthrDIDLinkedResourcesRegistry,
            resolver: EthrDidLinkedResourcesResolver::new(),
            signer,
        }
    }

    pub async fn create_resource(
        &self,
        did: &str,
        resource_input: ResourceInput,
    ) -> Result<Resource, Box<dyn Error>> {
        let resource = self
            .registry
            .create_or_update_resource(self.signer.clone(), &did, resource_input)
            .await?;

        let metadata_node = self
            .resolver
            .resolve_metadata_chain_node_for_event(&resource)
            .await;

        Ok(Resource::from((resource, metadata_node)))
    }
}
