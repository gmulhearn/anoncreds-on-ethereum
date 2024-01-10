pub mod anoncreds_primitives_ledger_data;
pub mod json_ledger_data;
pub mod status_list_update_ledger_data;

pub trait LedgerData {
    fn into_ledger_bytes(self) -> Vec<u8>;
    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized;
}
