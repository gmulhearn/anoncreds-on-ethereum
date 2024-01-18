#[derive(Clone, Debug, PartialEq)]
pub struct ContractNetworkConfig {
    pub contract_address: String,
    pub rpc_url: String,
    pub chain_id: u64,
}
