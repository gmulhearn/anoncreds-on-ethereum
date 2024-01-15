use ethers::types::H160;

const ETHR_DID_SUB_METHOD: &str = "local";

pub fn did_identity_as_full_did(address: &H160) -> String {
    // note that debug fmt of address is the '0x..' hex encoding.
    // where as .to_string() (fmt) truncates it
    format!("did:ethr:{ETHR_DID_SUB_METHOD}:{address:?}")
}

pub fn full_did_into_did_identity(did: &str) -> H160 {
    let identity_hex_str = did
        .split(":")
        .last()
        .expect(&format!("Could not read find identity of DID: {did}"));
    identity_hex_str.parse().unwrap()
}

pub fn extract_did_of_dlr_resource_uri(resource_uri: &str) -> String {
    resource_uri.split("/resources").next().unwrap().to_owned()
}
