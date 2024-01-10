use std::time::{SystemTime, UNIX_EPOCH};

use serde::{de::DeserializeOwned, Serialize};

pub fn get_epoch_secs() -> u64 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// a hacky way to clone when not possible
pub fn serde_clone<T: Serialize + DeserializeOwned>(data: &T) -> T {
    serde_json::from_str(&serde_json::to_string(data).unwrap()).unwrap()
}
