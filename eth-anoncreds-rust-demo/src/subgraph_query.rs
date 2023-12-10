use serde::Deserialize;
use serde_json::{json, Value};

const MOST_RECENT_STATUS_LIST_UPDATE_QUERY: &str = r#"
query MostRecentStatusListUpdate($revRegId: String, $lteTimestamp: Int) {
    statusListUpdateEvents(
      where: {blockTimestamp_lte: $lteTimestamp, revocationRegistryId: $revRegId}
      first: 1
      orderBy: blockTimestamp
      orderDirection: desc
    ) {
      id
      blockTimestamp
      revocationRegistryId
      statusList_currentAccumulator
      statusList_metadata_blockNumber
      statusList_metadata_blockTimestamp
      statusList_previousMetadata_blockTimestamp
      statusList_previousMetadata_blockNumber
      statusList_revocationList
    }
  }
  "#;
const MOST_RECENT_STATUS_LIST_UPDATE_OP_NAME: &str = "MostRecentStatusListUpdate";
const SUBGRAPH_API_URL: &str = "http://localhost:8000/subgraphs/name/anoncreds-registry-subgraph";

pub async fn get_status_list_event_most_recent_to(
    rev_reg_id: &str,
    timestamp: u64,
) -> MostRecentStatusListUpdateQueryResult {
    let request_body = json!({
        "operationName": MOST_RECENT_STATUS_LIST_UPDATE_OP_NAME,
        "query": MOST_RECENT_STATUS_LIST_UPDATE_QUERY,
        "variables": {
            "lteTimestamp": timestamp,
            "revRegId": rev_reg_id
        }
    });

    let res = reqwest::Client::default()
        .post(SUBGRAPH_API_URL)
        .json(&request_body)
        .send()
        .await
        .unwrap();

    let mut res = res.json::<Value>().await.unwrap();
    let item = res["data"]["statusListUpdateEvents"][0].take();

    serde_json::from_value(item).unwrap()
}

#[derive(Deserialize)]
pub struct MostRecentStatusListUpdateQueryResult {
    #[serde(rename = "revocationRegistryId")]
    pub rev_reg_id: String,
    #[serde(rename = "statusList_currentAccumulator")]
    pub current_accum: String,
    #[serde(rename = "statusList_revocationList")]
    pub status_list: String,
    #[serde(rename = "statusList_metadata_blockTimestamp")]
    pub timestamp: String,
}
