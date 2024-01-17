use serde::Deserialize;
use serde_json::{json, Value};

use crate::utils::full_did_into_did_identity;

const MOST_RECENT_RESOURCE_UPDATE_OP_NAME: &str = "ResourceForNameAndTypeAtTimestamp";
const MOST_RECENT_RESOURCE_UPDATE_QUERY: &str = r#"
query ResourceForNameAndTypeAtTimestamp($didIdentity: String, $resourceName: String, $resourceType: String, $timestamp: Int) {
    newResources(
      where: {blockTimestamp_lte: $timestamp, didIdentity: $didIdentity, resourceName: $resourceName, resourceType: $resourceType }
      first: 1
      orderBy: blockTimestamp
      orderDirection: desc
    ) {
        id
        blockNumber
        blockTimestamp
    	content
        didIdentity
	    resourceId
        resourceName
        resourceType
        resourceVersion
        resourceMediaType
        metadataChainNodeIndex
    }
  }
  "#;
const SUBGRAPH_API_URL: &str = "http://localhost:8000/subgraphs/name/example-subgraph";

pub async fn get_resource_event_most_recent_to(
    did: &str,
    resource_name: &str,
    resource_type: &str,
    timestamp: u64,
) -> Option<ResourceForNameAndTypeAtTimestampQueryResult> {
    let did_identity = full_did_into_did_identity(did);

    let request_body = json!({
        "operationName": MOST_RECENT_RESOURCE_UPDATE_OP_NAME,
        "query": MOST_RECENT_RESOURCE_UPDATE_QUERY,
        "variables": {
            "didIdentity": did_identity,
            "resourceName": resource_name,
            "resourceType": resource_type,
            "timestamp": timestamp,
        }
    });

    let res = reqwest::Client::default()
        .post(SUBGRAPH_API_URL)
        .json(&request_body)
        .send()
        .await
        .unwrap();

    let mut res = res.json::<Value>().await.unwrap();
    let item = res["data"]["newResources"].get_mut(0).take();

    item.map(|i| serde_json::from_value(i.take()).unwrap())
}

#[derive(Deserialize)]
pub struct ResourceForNameAndTypeAtTimestampQueryResult {
    #[serde(rename = "blockTimestamp")]
    pub block_timestamp: String,
    #[serde(rename = "content")]
    pub content: String,
    #[serde(rename = "didIdentity")]
    pub did_identity: String,
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    #[serde(rename = "resourceName")]
    pub resource_name: String,
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "resourceVersion")]
    pub resource_version: String,
    #[serde(rename = "resourceMediaType")]
    pub resource_media_type: String,
    #[serde(rename = "metadataChainNodeIndex")]
    pub metadata_chain_node_index: String,
}
