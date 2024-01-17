use std::error::Error;

use did_ethr_linked_resources::config::{ContractNetworkConfig, DidEthrSubMethod};

const RPC_URL_ENV_VAR: &str = "RPC_URL";
const CHAIN_ID_ENV_VAR: &str = "CHAIN_ID";
const DID_ETHR_CONTRACT_ADDRESS_ENV_VAR: &str = "DID_ETHR_CONTRACT_ADDRESS";
const DLR_CONTRACT_ADDRESS_ENV_VAR: &str = "DLR_CONTRACT_ADDRESS";

const DEFAULT_RPC_URL: &str = "http://localhost:8545";
const DEFAULT_CHAIN_ID: u64 = 31337;
const DEFAULT_DID_ETHR_CONTRACT_ADDRESS: &str = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
const DEFAULT_DLR_CONTRACT_ADDRESS: &str = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";
const DEFAULT_DID_ETHR_SUB_METHOD: &str = "local";

const CHAIN_ID_TO_KNOWN_SUB_METHOD: [(u64, &str); 7] = [
    (1, "mainnet"),
    (3, "ropsten"),
    (4, "rinkeby"),
    (5, "goerli"),
    (42, "kovan"),
    (137, "polygon"),
    (31337, "local"),
];

pub struct DemoConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub did_ethr_contract_address: String,
    pub dlr_contract_address: String,
    pub did_ethr_sub_method: DidEthrSubMethod,
}

fn sub_method_name_from_chain_id(chain_id: u64) -> DidEthrSubMethod {
    if let Some((_, sub_method)) = CHAIN_ID_TO_KNOWN_SUB_METHOD
        .iter()
        .find(|(id, _)| *id == chain_id)
    {
        return DidEthrSubMethod(sub_method.to_string());
    }

    let hex_chain_id = format!("0x{:x}", chain_id);
    DidEthrSubMethod(hex_chain_id)
}

impl DemoConfig {
    /// load from env, else local
    pub fn load() -> Self {
        match Self::try_from_env() {
            Ok(c) => {
                println!("Loaded config from env");
                c
            }
            Err(e) => {
                println!("Failed to load config from env: {}", e);
                println!("Loading local config");
                Self::local()
            }
        }
    }

    fn local() -> Self {
        Self {
            rpc_url: DEFAULT_RPC_URL.to_string(),
            chain_id: DEFAULT_CHAIN_ID,
            did_ethr_contract_address: DEFAULT_DID_ETHR_CONTRACT_ADDRESS.to_string(),
            dlr_contract_address: DEFAULT_DLR_CONTRACT_ADDRESS.to_string(),
            did_ethr_sub_method: DidEthrSubMethod(DEFAULT_DID_ETHR_SUB_METHOD.to_string()),
        }
    }

    fn try_from_env() -> Result<Self, Box<dyn Error>> {
        dotenv::dotenv().ok();

        let rpc_url = std::env::var(RPC_URL_ENV_VAR)?;
        let chain_id = std::env::var(CHAIN_ID_ENV_VAR)?.parse()?;
        let did_ethr_contract_address = std::env::var(DID_ETHR_CONTRACT_ADDRESS_ENV_VAR)?;
        let dlr_contract_address = std::env::var(DLR_CONTRACT_ADDRESS_ENV_VAR)?;
        let did_ethr_sub_method = sub_method_name_from_chain_id(chain_id);

        Ok(Self {
            rpc_url,
            chain_id,
            did_ethr_contract_address,
            dlr_contract_address,
            did_ethr_sub_method,
        })
    }

    pub fn get_did_ethr_network_config(&self) -> ContractNetworkConfig {
        ContractNetworkConfig {
            rpc_url: self.rpc_url.clone(),
            contract_address: self.did_ethr_contract_address.clone(),
            did_ethr_sub_method: self.did_ethr_sub_method.clone(),
        }
    }

    pub fn get_dlr_network_config(&self) -> ContractNetworkConfig {
        ContractNetworkConfig {
            rpc_url: self.rpc_url.clone(),
            contract_address: self.dlr_contract_address.clone(),
            did_ethr_sub_method: self.did_ethr_sub_method.clone(),
        }
    }
}
