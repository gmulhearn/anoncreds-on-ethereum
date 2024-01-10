use serde::Deserialize;
use serde_json::{json, Value};

use super::did_linked_resource_id::DIDLinkedResourceId;

const MOST_RECENT_RESOURCE_UPDATE_OP_NAME: &str = "MostRecentResourceUpdate";
const MOST_RECENT_RESOURCE_UPDATE_QUERY: &str = r#"
query MostRecentResourceUpdate($didIdentity: String, $name: String, $lteTimestamp: Int) {
    mutableResourceUpdatedEvents(
      where: {blockTimestamp_lte: $lteTimestamp, didIdentity: $didIdentity, name: $name }
      first: 1
      orderBy: blockTimestamp
      orderDirection: desc
    ) {
      id
      blockNumber
      blockTimestamp
      didIdentity
      name
      resource_content
      resource_metadata_blockNumber
      resource_metadata_blockTimestamp
      resource_previousMetadata_blockNumber
      resource_previousMetadata_blockTimestamp
    }
  }
  "#;
const SUBGRAPH_API_URL: &str = "http://localhost:8000/subgraphs/name/example-subgraph";

pub async fn get_resource_update_event_most_recent_to(
    resource_id: DIDLinkedResourceId,
    timestamp: u64,
) -> MostRecentResourceUpdateQueryResult {
    let did_identity = resource_id.did_identity;
    let resource_name = resource_id.resource_name;

    let request_body = json!({
        "operationName": MOST_RECENT_RESOURCE_UPDATE_OP_NAME,
        "query": MOST_RECENT_RESOURCE_UPDATE_QUERY,
        "variables": {
            "didIdentity": did_identity,
            "name": resource_name,
            "lteTimestamp": timestamp,
        }
    });

    let res = reqwest::Client::default()
        .post(SUBGRAPH_API_URL)
        .json(&request_body)
        .send()
        .await
        .unwrap();

    let mut res = res.json::<Value>().await.unwrap();
    let item = res["data"]["mutableResourceUpdatedEvents"][0].take();

    serde_json::from_value(item).unwrap()
}

#[derive(Deserialize)]
pub struct MostRecentResourceUpdateQueryResult {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "didIdentity")]
    pub did_identity: String,
    #[serde(rename = "resource_content")]
    pub content_hex: String,
    #[serde(rename = "blockTimestamp")]
    pub timestamp: String,
}
