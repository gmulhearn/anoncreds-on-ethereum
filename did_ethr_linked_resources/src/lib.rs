pub mod contracts;
pub mod registrar;
pub mod resolver;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::{
        contracts::test_utils::get_writer_ethers_client,
        registrar::EthrDidLinkedResourcesRegistrar, resolver::EthrDidLinkedResourcesResolver,
        types::input::ResourceInput, utils::did_identity_as_full_did,
    };

    #[tokio::test]
    async fn test_resolve_versions_by_time() {
        let signer = get_writer_ethers_client(0);
        let did = did_identity_as_full_did(&signer.address());

        let resolver = EthrDidLinkedResourcesResolver::new();
        let registrar = EthrDidLinkedResourcesRegistrar::new(signer);

        let resource_name = &format!("foo{}", uuid::Uuid::new_v4());
        let resource_type = "bar";

        // create resources
        let mut res_input = ResourceInput {
            resource_name: resource_name.to_owned(),
            resource_type: resource_type.to_owned(),
            content: String::from("hello world").into_bytes(),
            ..Default::default()
        };
        let created_res1 = registrar
            .create_resource(&did, res_input.clone())
            .await
            .unwrap();

        res_input.content = String::from("hello world 2").into_bytes();
        let created_res2 = registrar
            .create_resource(&did, res_input.clone())
            .await
            .unwrap();

        res_input.content = String::from("hello world 3").into_bytes();
        let created_res3 = registrar
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
