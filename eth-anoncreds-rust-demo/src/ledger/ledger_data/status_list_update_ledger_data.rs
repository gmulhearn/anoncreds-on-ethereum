use anoncreds::data_types::{issuer_id::IssuerId, rev_status_list::serde_revocation_list};
use bitvec::vec::BitVec;
use serde_json::{json, Value};
use ursa::pair::PointG2;

use crate::ledger::did_parsing_helpers::extract_did_of_dlr_resource_uri;

use super::LedgerDataTransformer;

#[derive(Debug, Clone)]
pub struct StatusListUpdateLedgerData {
    pub accumulator: PointG2,
    pub rev_list: BitVec,
}

impl LedgerDataTransformer for StatusListUpdateLedgerData {
    // ledger data: [accumulator (128bytes), rev_list (variable)]

    fn into_ledger_bytes(self) -> Vec<u8> {
        let accumulator_as_bytes = self.accumulator.to_bytes().unwrap();
        let rev_list_as_bytes = bitvec_to_bytes(self.rev_list);
        [accumulator_as_bytes, rev_list_as_bytes].concat()
    }

    fn from_ledger_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        let accumulator_bytes = bytes[..PointG2::BYTES_REPR_SIZE].to_vec();
        let rev_list_bytes = bytes[PointG2::BYTES_REPR_SIZE..].to_vec();
        let accumulator = PointG2::from_bytes(&accumulator_bytes).unwrap();
        let rev_list = bytes_to_bitvec(rev_list_bytes);

        Self {
            accumulator,
            rev_list,
        }
    }
}

impl StatusListUpdateLedgerData {
    pub fn from_anoncreds_data(anoncreds_data: &anoncreds::types::RevocationStatusList) -> Self {
        // dismantle the inner parts that we can't access
        let revocation_status_list_json: Value = serde_json::to_value(anoncreds_data).unwrap();
        let current_accumulator = revocation_status_list_json
            .get("currentAccumulator")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        let accumulator = PointG2::from_string(&current_accumulator).unwrap();

        let revocation_list_val = revocation_status_list_json.get("revocationList").unwrap();
        let rev_list = serde_revocation_list::deserialize(revocation_list_val).unwrap();

        Self {
            accumulator,
            rev_list,
        }
    }

    pub fn into_anoncreds_data(
        self,
        timestamp: u64,
        rev_reg_id: &str,
    ) -> anoncreds::types::RevocationStatusList {
        let issuer_did = extract_did_of_dlr_resource_uri(rev_reg_id);

        let current_accumulator_str = self.accumulator.to_string().unwrap();
        let current_accumulator = serde_json::from_value(json!(&current_accumulator_str)).unwrap();

        anoncreds::types::RevocationStatusList::new(
            Some(rev_reg_id),
            IssuerId::try_from(issuer_did).unwrap(),
            self.rev_list,
            Some(current_accumulator),
            Some(timestamp.into()),
        )
        .unwrap()
    }
}

fn bitvec_to_bytes(bitvec: BitVec) -> Vec<u8> {
    let mut bitvec_as_u8_array = vec![0; (bitvec.len() / 8) + 1];

    for (idx, bit) in bitvec.into_iter().enumerate() {
        let byte = idx / 8;
        let shift = 7 - idx % 8;
        bitvec_as_u8_array[byte] |= (bit as u8) << shift;
    }

    bitvec_as_u8_array
}

fn bytes_to_bitvec(bytes: Vec<u8>) -> BitVec {
    let rev_list: BitVec<_> = BitVec::from_vec(bytes);
    rev_list.into_iter().collect()
}
