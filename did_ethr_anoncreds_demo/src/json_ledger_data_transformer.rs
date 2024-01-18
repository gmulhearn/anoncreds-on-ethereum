use serde::{de::DeserializeOwned, Serialize};

use did_ethr_anoncreds::ledger_data_transformer::LedgerDataTransformer;

pub struct JsonLedgerData<T: Serialize + DeserializeOwned>(pub T);

impl<T: Serialize + DeserializeOwned> LedgerDataTransformer for JsonLedgerData<T> {
    fn into_ledger_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap()
    }

    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        Self(serde_json::from_slice(bytes).unwrap())
    }
}
