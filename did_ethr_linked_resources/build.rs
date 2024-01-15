use ethers::prelude::Abigen;
use std::{env, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    // gen types for ethereum did registry

    let abi_source =
        "../smart-contracts/artifacts/contracts/EthereumDIDRegistry.sol/EthereumDIDRegistry.json";
    let out_file = Path::new(&out_dir).join("ethereum_did_registry_contract.rs");
    if out_file.exists() {
        std::fs::remove_file(&out_file).unwrap();
    }

    Abigen::new("EthereumDIDRegistry", abi_source)
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file(out_file)
        .unwrap();

    // gen types for EthrDIDLinkedResourcesRegistry.sol

    let abi_source =
        "../smart-contracts/artifacts/contracts/EthrDIDLinkedResourcesRegistry.sol/EthrDIDLinkedResourcesRegistry.json";
    let out_file = Path::new(&out_dir).join("ethr_dlr_registry_contract.rs");
    if out_file.exists() {
        std::fs::remove_file(&out_file).unwrap();
    }

    Abigen::new("EthrDLRRegistry", abi_source)
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file(out_file)
        .unwrap();
}
