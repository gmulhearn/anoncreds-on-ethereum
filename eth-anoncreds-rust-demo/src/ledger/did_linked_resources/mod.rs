pub mod registar;
pub mod resolver;
pub mod types;

mod helper {
    use chrono::{TimeZone, Utc};

    use crate::ledger::{
        contracts::ethr_dlr_registry::{NewResourceFilter, ResourceVersionMetadataChainNode},
        did_linked_resources::types::output::Resource,
    };

    impl From<(NewResourceFilter, ResourceVersionMetadataChainNode)> for Resource {
        fn from(
            (event, metadata_node): (NewResourceFilter, ResourceVersionMetadataChainNode),
        ) -> Self {
            let ledger_resource = event.resource;
            let ledger_res_meta = ledger_resource.metadata;

            let did_identity = event.did_identity;

            let resource_uri = format!(
                "did:local:ethr:{did_identity:?}/resources/{resource_id}",
                resource_id = ledger_resource.resource_id
            );

            let created_epoch = ledger_res_meta.created.block_timestamp;

            let previous_version_id = match metadata_node.previous_resource_id.to_string().as_str()
            {
                "0" => None,
                x => Some(x.to_owned()),
            };

            let next_version_id = match metadata_node.next_resource_id.to_string().as_str() {
                "0" => None,
                x => Some(x.to_owned()),
            };

            Resource {
                resource_uri,
                resource_type: ledger_res_meta.resource_type,
                resource_name: ledger_res_meta.resource_name,
                resource_id: Some(ledger_resource.resource_id.to_string()),
                resource_collection_id: Some(format!("{did_identity:?}")),
                resource_version_id: Some(ledger_res_meta.resource_version),
                media_type: ledger_res_meta.media_type,
                created: Utc.timestamp_opt(created_epoch as i64, 0).unwrap(),
                checksum: None,
                previous_version_id: previous_version_id,
                next_version_id: next_version_id,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::ledger::{
        contracts::get_writer_ethers_client,
        did_linked_resource_id::did_identity_as_full_did,
        did_linked_resources::{
            registar::EthrDidLinkedResourcesRegistar, resolver::EthrDidLinkedResourcesResolver,
            types::input::ResourceInput,
        },
    };

    #[tokio::test]
    async fn test_resolve_versions_by_time() {
        let signer = get_writer_ethers_client(0);
        let did = did_identity_as_full_did(&signer.address());

        let resolver = EthrDidLinkedResourcesResolver::new();
        let registar = EthrDidLinkedResourcesRegistar::new(signer);
        let resource_name = &format!("foo{}", uuid::Uuid::new_v4());
        let resource_type = "bar";

        // create resources
        let mut res_input = ResourceInput {
            resource_name: resource_name.to_owned(),
            resource_type: resource_type.to_owned(),
            content: String::from("hello world").into_bytes(),
            ..Default::default()
        };
        let created_res1 = registar
            .create_resource(&did, res_input.clone())
            .await
            .unwrap();

        res_input.content = String::from("hello world 2").into_bytes();
        let created_res2 = registar
            .create_resource(&did, res_input.clone())
            .await
            .unwrap();

        res_input.content = String::from("hello world 3").into_bytes();
        let created_res3 = registar
            .create_resource(&did, res_input.clone())
            .await
            .unwrap();

        // resolve exact
        let resolved_res = resolver
            .resolve_query(&created_res2.resource_uri)
            .await
            .unwrap();

        dbg!(resolved_res);

        // resolve over time range
        let epoch_range =
            created_res1.created.timestamp() - 4..created_res3.created.timestamp() + 4;
        for epoch in epoch_range.step_by(2) {
            // get time as DateTime string
            let datetime = Utc.timestamp_opt(epoch, 0).unwrap();
            let formatted_dt = datetime.to_rfc3339();
            let version_time = urlencoding::encode(&formatted_dt);
            let query = format!(
                "{did}?resourceName={resource_name}&resourceType={resource_type}&versionTime={version_time}"
            );

            let resolved_res = resolver.resolve_query(&query).await.ok();
            dbg!(formatted_dt, resolved_res);
        }
    }
}
