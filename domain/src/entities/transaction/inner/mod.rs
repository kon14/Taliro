use crate::entities::transaction::{TransactionInput, TransactionOutput};
use crate::ext::AppErrorConvertibleDomain;
use crate::types::hash::{Hash, TryHashable};
use crate::types::time::DateTime;
use bincode::{Decode, Encode};
use blake2::{Blake2b512, Digest};
use common::error::AppError;

#[derive(Clone, Debug, Encode, Decode)]
pub(crate) struct InnerTransaction {
    pub(super) inputs: Vec<TransactionInput>,
    pub(super) outputs: Vec<TransactionOutput>,
    pub(super) timestamp: DateTime,
}

impl TryHashable for InnerTransaction {
    fn try_hash(&self) -> Result<Hash, AppError> {
        let config = bincode::config::standard();
        let serialized_bytes = bincode::encode_to_vec(self, config).to_app_error()?;

        let mut hasher = Blake2b512::new();
        hasher.update(&serialized_bytes);
        let result = hasher.finalize();
        let bytes = result.as_slice();

        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&bytes[..32]);
        let hash = Hash::new(hash_bytes);
        Ok(hash)
    }
}
