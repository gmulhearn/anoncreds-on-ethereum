#[derive(Clone, Debug, PartialEq)]
pub struct ContractNetworkConfig {
    pub contract_address: String,
    pub rpc_url: String,
    pub did_ethr_sub_method: DidEthrSubMethod,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DidEthrSubMethod(pub String);
