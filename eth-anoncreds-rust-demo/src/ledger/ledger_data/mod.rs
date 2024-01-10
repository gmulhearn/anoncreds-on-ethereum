pub mod anoncreds_primitives_ledger_data_transformers;
pub mod json_ledger_data_transformer;
pub mod status_list_update_ledger_data;

pub trait LedgerDataTransformer {
    fn into_ledger_bytes(self) -> Vec<u8>;
    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized;
}
