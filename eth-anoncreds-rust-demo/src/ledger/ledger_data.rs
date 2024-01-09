pub trait LedgerData {
    fn into_ledger_bytes(self) -> Vec<u8>;
    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized;
}
