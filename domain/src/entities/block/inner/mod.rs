use crate::entities::block::{BlockDifficultyTarget, BlockHeight, BlockNonce};
use crate::entities::transaction::{NonValidatedTransaction, Transaction, TransactionsMerkleRoot};
use crate::ext::AppErrorConvertibleDomain;
use crate::types::hash::{Hash, TryHashable};
use crate::types::time::DateTime;
use bincode::{Decode, Encode};
use blake2::{Blake2b512, Digest};
use common::error::AppError;

#[derive(Clone, Debug, Encode, Decode)]
pub(super) struct InnerBlock {
    pub(super) height: BlockHeight,
    pub(super) prev_block_hash: Option<Hash>,
    pub(super) nonce: BlockNonce,
    pub(super) difficulty_target: BlockDifficultyTarget,
    pub(super) transactions_merkle_root: TransactionsMerkleRoot,
    pub(super) transactions: Vec<Transaction>,
    pub(super) timestamp: DateTime,
}

#[derive(Clone, Debug, Encode, Decode)]
pub(super) struct NonValidatedInnerBlock {
    pub(super) height: BlockHeight,
    pub(super) prev_block_hash: Option<Hash>,
    pub(super) nonce: BlockNonce,
    pub(super) difficulty_target: BlockDifficultyTarget,
    pub(super) transactions_merkle_root: TransactionsMerkleRoot,
    pub(super) transactions: Vec<NonValidatedTransaction>,
    pub(super) timestamp: DateTime,
}

impl InnerBlock {
    pub(super) fn invalidate(self) -> NonValidatedInnerBlock {
        NonValidatedInnerBlock {
            height: self.height,
            prev_block_hash: self.prev_block_hash,
            nonce: self.nonce,
            difficulty_target: self.difficulty_target,
            transactions_merkle_root: self.transactions_merkle_root,
            transactions: self
                .transactions
                .into_iter()
                .map(|tx| tx.invalidate())
                .collect(),
            timestamp: self.timestamp,
        }
    }

    /// Internal method to construct a validated [`InnerBlock`] from a [`NonValidatedInnerBlock`].<br />
    /// Called exclusively post-validation.
    pub(super) fn _new_validated(inner: NonValidatedInnerBlock) -> InnerBlock {
        let transactions = inner
            .transactions
            .into_iter()
            .map(Transaction::_new_validated)
            .collect();
        Self {
            height: inner.height,
            prev_block_hash: inner.prev_block_hash,
            nonce: inner.nonce,
            difficulty_target: inner.difficulty_target,
            transactions_merkle_root: inner.transactions_merkle_root,
            transactions,
            timestamp: inner.timestamp,
        }
    }
}

impl InnerBlock {
    pub(super) fn is_genesis_block(&self) -> bool {
        self.height.0 == u64::MIN
    }
}

impl NonValidatedInnerBlock {
    pub(super) fn is_genesis_block(&self) -> bool {
        self.height.0 == u64::MIN
    }
}

impl TryHashable for InnerBlock {
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

impl TryHashable for NonValidatedInnerBlock {
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
