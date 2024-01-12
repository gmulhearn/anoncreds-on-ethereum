use std::error::Error;

use chrono::offset::Utc;
use chrono::DateTime;
use ethers::types::H160;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceQueryParameters {
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub resource_type: Option<String>,
    pub resource_version_id: Option<String>,
    pub version_time: Option<DateTime<Utc>>,
    pub version_id: Option<String>, // what's the difference to resource_version_id?
    pub linked_resource: Option<bool>,
    pub resource_metadata: Option<bool>,
    pub latest_resource_version: Option<bool>,
    pub all_resource_versions: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceQuery {
    pub did_identity: H160,
    pub parameters: ResourceQueryParameters,
}

impl ResourceQuery {
    pub fn parse_from_str(did_query: &str) -> Result<Self, Box<dyn Error>> {
        let mut query_params = ResourceQueryParameters {
            resource_id: None,
            resource_name: None,
            resource_type: None,
            resource_version_id: None,
            version_time: None,
            version_id: None,
            linked_resource: None,
            resource_metadata: None,
            latest_resource_version: None,
            all_resource_versions: None,
        };

        let did_query_url = Url::parse(did_query)?;

        if did_query_url.scheme() != "did" {
            return Err(format!("Invalid DID query: {}", did_query).into());
        }

        let did_query_path = did_query_url.path();
        let mut did_query_path_parts = did_query_path.split("/");
        let method_and_did = did_query_path_parts
            .next()
            .ok_or("Could not parse DID query: missing method and DID")?;
        // TODO - assert ethr method?
        let did_identity_hex_str = method_and_did
            .split(":")
            .last()
            .ok_or(format!("Could not read find author of DID: {did_query}"))?;
        let did_identity = did_identity_hex_str.parse().unwrap();

        match (
            did_query_path_parts.next(),
            did_query_path_parts.next(),
            did_query_path_parts.next(),
        ) {
            (None, _, _) => {}
            (Some("resources"), Some(resource_id), None) => {
                query_params.resource_id = Some(resource_id.to_owned());
            }
            _ => {
                return Err(format!("Invalid DID query: {}", did_query).into());
            }
        }

        for (name, value) in did_query_url.query_pairs() {
            match name.as_ref() {
                "resourceId" => query_params.resource_id = Some(value.into_owned()),
                "resourceName" => query_params.resource_name = Some(value.into_owned()),
                "resourceType" => query_params.resource_type = Some(value.into_owned()),
                "resourceVersionId" => query_params.resource_version_id = Some(value.into_owned()),
                "versionTime" => query_params.version_time = Some(value.parse::<DateTime<Utc>>()?),
                "versionId" => query_params.version_id = Some(value.into_owned()),
                "linkedResource" => query_params.linked_resource = Some(value.parse::<bool>()?),
                "resourceMetadata" => query_params.resource_metadata = Some(value.parse::<bool>()?),
                "latestResourceVersion" => {
                    query_params.latest_resource_version = Some(value.parse::<bool>()?)
                }
                "allResourceVersions" => {
                    query_params.all_resource_versions = Some(value.parse::<bool>()?)
                }
                _ => return Err(format!("Unknown query parameter: {}", name).into()),
            }
        }

        Ok(ResourceQuery {
            did_identity,
            parameters: query_params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceQuery;

    #[test]
    fn test_fully_loaded_query() {
        let query = "did:ethr:local:0x1234567890123456789012345678901234567890?resourceId=123&resourceName=456&resourceType=789&resourceVersionId=012&versionTime=2021-01-01T00:00:00Z&versionId=345&linkedResource=true&resourceMetadata=true&latestResourceVersion=true&allResourceVersions=true";

        let query = ResourceQuery::parse_from_str(query).unwrap();

        assert_eq!(
            query.did_identity,
            "0x1234567890123456789012345678901234567890"
                .parse()
                .unwrap()
        );
        assert_eq!(query.parameters.resource_id.unwrap(), "123");
        assert_eq!(query.parameters.resource_name.unwrap(), "456");
        assert_eq!(query.parameters.resource_type.unwrap(), "789");
        assert_eq!(query.parameters.resource_version_id.unwrap(), "012");
        assert_eq!(
            query.parameters.version_time.unwrap().to_rfc3339(),
            "2021-01-01T00:00:00+00:00"
        );
        assert_eq!(query.parameters.version_id.unwrap(), "345");
        assert_eq!(query.parameters.linked_resource.unwrap(), true);
        assert_eq!(query.parameters.resource_metadata.unwrap(), true);
        assert_eq!(query.parameters.latest_resource_version.unwrap(), true);
        assert_eq!(query.parameters.all_resource_versions.unwrap(), true);
    }

    #[test]
    fn test_direct_resource_query() {
        let query = "did:ethr:local:0x1234567890123456789012345678901234567890/resources/123";

        let query = ResourceQuery::parse_from_str(query).unwrap();

        assert_eq!(
            query.did_identity,
            "0x1234567890123456789012345678901234567890"
                .parse()
                .unwrap()
        );
        assert_eq!(query.parameters.resource_id.unwrap(), "123");
        assert!(query.parameters.resource_name.is_none());
        assert!(query.parameters.resource_type.is_none());
        assert!(query.parameters.resource_version_id.is_none());
        assert!(query.parameters.version_time.is_none());
        assert!(query.parameters.version_id.is_none());
        assert!(query.parameters.linked_resource.is_none());
        assert!(query.parameters.resource_metadata.is_none());
        assert!(query.parameters.latest_resource_version.is_none());
        assert!(query.parameters.all_resource_versions.is_none());
    }

    #[test]
    fn test_some_spec_queries() {
        // https://wiki.trustoverip.org/display/HOME/DID-Linked+Resources+Specification
        let query = ResourceQuery::parse_from_str("did:ethr:0x1234567890123456789012345678901234567890?resourceName=degreeLaw&resourceVersionId=1.3.1").unwrap();
        assert_eq!(
            query.did_identity,
            "0x1234567890123456789012345678901234567890"
                .parse()
                .unwrap()
        );
        assert_eq!(query.parameters.resource_name.unwrap(), "degreeLaw");
        assert_eq!(query.parameters.resource_version_id.unwrap(), "1.3.1");

        let query = ResourceQuery::parse_from_str("did:ethr:0x1234567890123456789012345678901234567890?resourceName=degreeLaw&resourceType=JSONSchema2020&versionTime=2015-03-11T05:30:02Z").unwrap();
        assert_eq!(
            query.did_identity,
            "0x1234567890123456789012345678901234567890"
                .parse()
                .unwrap()
        );
        assert_eq!(query.parameters.resource_name.unwrap(), "degreeLaw");
        assert_eq!(query.parameters.resource_type.unwrap(), "JSONSchema2020");
        assert_eq!(
            query.parameters.version_time.unwrap().to_rfc3339(),
            "2015-03-11T05:30:02+00:00"
        );
        assert!(query.parameters.resource_metadata.is_none());

        let query = ResourceQuery::parse_from_str("did:ethr:0x1234567890123456789012345678901234567890?resourceName=degreeLaw&resourceType=JSONSchema2020&versionTime=2018-07-19T08:40:00Z&resourceMetadata=true").unwrap();   
        assert_eq!(
            query.did_identity,
            "0x1234567890123456789012345678901234567890"
                .parse()
                .unwrap()
        );
        assert_eq!(query.parameters.resource_name.unwrap(), "degreeLaw");
        assert_eq!(query.parameters.resource_type.unwrap(), "JSONSchema2020");
        assert_eq!(
            query.parameters.version_time.unwrap().to_rfc3339(),
            "2018-07-19T08:40:00+00:00"
        );
        assert_eq!(query.parameters.resource_metadata.unwrap(), true);
    }
}
