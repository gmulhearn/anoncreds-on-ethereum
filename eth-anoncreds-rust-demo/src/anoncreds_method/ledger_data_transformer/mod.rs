pub mod anoncreds_primitives_ledger_data_transformers;
pub mod json_ledger_data_transformer;
pub mod status_list_update_ledger_data;

pub trait LedgerDataTransformer {
    fn into_ledger_bytes(self) -> Vec<u8>;
    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized;
}

// https://docs.cheqd.io/identity/advanced/anoncreds/schema
pub const SCHEMA_RESOURCE_TYPE: &str = "anonCredsSchema";
// https://docs.cheqd.io/identity/advanced/anoncreds/credential-definition
pub const CRED_DEF_RESOURCE_TYPE: &str = "anonCredsCredDef";
// https://docs.cheqd.io/identity/advanced/anoncreds/revocation-registry-definition
pub const REV_REG_DEF_RESOURCE_TYPE: &str = "anonCredsRevocRegDef";
// https://docs.cheqd.io/identity/advanced/anoncreds/revocation-status-list
pub const STATUS_LIST_RESOURCE_TYPE: &str = "anonCredsStatusList";

pub const BINARY_MEDIA_TYPE: &str = "application/octet-stream";

pub const NO_VERSION: &str = "";
