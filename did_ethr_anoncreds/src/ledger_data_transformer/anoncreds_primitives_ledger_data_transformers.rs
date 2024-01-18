//! Define ledger data conversions for anoncreds primitives.
//! Currently just using serde JSON form. but could be more optimized.

use super::LedgerDataTransformer;

impl LedgerDataTransformer for anoncreds::data_types::schema::Schema {
    fn into_ledger_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        serde_json::from_slice(bytes).unwrap()
    }
}

impl LedgerDataTransformer for anoncreds::data_types::cred_def::CredentialDefinition {
    fn into_ledger_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        serde_json::from_slice(bytes).unwrap()
    }
}

impl LedgerDataTransformer for anoncreds::data_types::rev_reg_def::RevocationRegistryDefinition {
    fn into_ledger_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        serde_json::from_slice(bytes).unwrap()
    }
}
