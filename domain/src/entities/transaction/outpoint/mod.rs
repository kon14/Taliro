use crate::encode::{TryDecode, TryEncode};
use crate::ext::AppErrorConvertibleDomain;
use crate::types::hash::Hash;
use bincode::{Decode, Encode};
use common::error::AppError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct TransactionOutPoint {
    tx_id: Hash,
    tx_output_index: usize,
}

impl TransactionOutPoint {
    pub fn new(tx_id: Hash, tx_output_index: usize) -> Self {
        Self {
            tx_id,
            tx_output_index,
        }
    }

    pub fn get_tx_id(&self) -> &Hash {
        &self.tx_id
    }

    pub fn get_tx_output_index(&self) -> usize {
        self.tx_output_index
    }
}

impl TryEncode for TransactionOutPoint {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for TransactionOutPoint {
    fn try_decode(data: &[u8]) -> Result<Self, AppError> {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}

impl std::fmt::Display for TransactionOutPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TransactionOutput(Hash({})[{}])",
            self.tx_id, self.tx_output_index
        )
    }
}
