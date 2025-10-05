mod difficulty;
mod height;
mod inner;
mod nonce;
mod template;

pub use difficulty::BlockDifficultyTarget;
pub use height::BlockHeight;
pub use nonce::BlockNonce;
pub use template::BlockTemplate;

use crate::encode::{TryDecode, TryEncode};
use crate::entities::block::inner::{InnerBlock, NonValidatedInnerBlock};
use crate::entities::transaction::{
    NonValidatedTransaction, Transaction, TransactionOutput, TransactionsMerkleRoot,
};
use crate::ext::AppErrorConvertibleDomain;
use crate::types::hash::{Hash, TryHashable};
use crate::types::time::DateTime;
use bincode::{Decode, Encode};
use common::error::AppError;

#[derive(Clone, Debug, Encode, Decode)]
pub struct Block {
    hash: Hash,
    data: InnerBlock,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct NonValidatedBlock {
    hash: Hash,
    data: NonValidatedInnerBlock,
}

impl Block {
    /// Flips a validated [`Block`] into a [`NonValidatedBlock`].<br />
    pub fn invalidate(self) -> NonValidatedBlock {
        NonValidatedBlock {
            hash: self.hash,
            data: self.data.invalidate(),
        }
    }

    /// Internal method to construct a validated [`Block`] from a [`NonValidatedBlock`].<br />
    /// Called exclusively post-validation.
    pub(crate) fn _new_validated(block: NonValidatedBlock) -> Block {
        Self {
            hash: block.hash,
            data: InnerBlock::_new_validated(block.data),
        }
    }
}

impl NonValidatedBlock {
    pub(crate) fn new_genesis(
        cfg: crate::genesis::config::GenesisConfig,
    ) -> Result<Self, AppError> {
        let transactions = cfg
            .utxos
            .iter()
            .map(|utxo| {
                let tx_output = TransactionOutput::new((&utxo.wallet_pub_key).into(), utxo.amount);
                NonValidatedTransaction::new(vec![], vec![tx_output], cfg.timestamp.clone())
            })
            .collect::<Result<Vec<NonValidatedTransaction>, AppError>>()?;
        let transactions_merkle_root = TransactionsMerkleRoot::new_non_validated(&transactions)?;
        let data = NonValidatedInnerBlock {
            height: BlockHeight::genesis(),
            prev_block_hash: None,
            nonce: BlockNonce::default(),
            difficulty_target: BlockDifficultyTarget(0),
            transactions_merkle_root,
            transactions,
            timestamp: cfg.timestamp,
        };
        let hash = data.try_hash()?;
        let block = Self { hash, data };
        Ok(block)
    }

    pub(crate) fn from_template(block_tpl: BlockTemplate) -> Result<Self, AppError> {
        let transactions_merkle_root =
            TransactionsMerkleRoot::new_non_validated(&block_tpl.transactions)?;
        let data = NonValidatedInnerBlock {
            height: block_tpl.block_height,
            prev_block_hash: block_tpl.prev_block_hash,
            nonce: block_tpl.nonce,
            difficulty_target: block_tpl.difficulty_target,
            transactions_merkle_root,
            transactions: block_tpl.transactions,
            timestamp: DateTime::now(),
        };
        let hash = data.try_hash()?;
        let block = Self { hash, data };
        Ok(block)
    }
}

impl Block {
    pub fn get_hash(&self) -> Hash {
        self.hash.clone()
    }

    pub fn get_height(&self) -> BlockHeight {
        self.data.height.clone()
    }

    pub fn get_prev_block_hash(&self) -> Option<Hash> {
        self.data.prev_block_hash.clone()
    }

    pub fn get_nonce(&self) -> BlockNonce {
        self.data.nonce.clone()
    }

    pub fn get_difficulty_target(&self) -> BlockDifficultyTarget {
        self.data.difficulty_target.clone()
    }

    pub fn get_transactions_merkle_root(&self) -> &TransactionsMerkleRoot {
        &self.data.transactions_merkle_root
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.data.transactions
    }

    pub fn get_timestamp(&self) -> DateTime {
        self.data.timestamp.clone()
    }

    pub fn is_genesis_block(&self) -> bool {
        self.data.is_genesis_block()
    }
}

impl NonValidatedBlock {
    pub fn get_hash(&self) -> Hash {
        self.hash.clone()
    }

    pub fn get_height(&self) -> BlockHeight {
        self.data.height.clone()
    }

    pub fn get_prev_block_hash(&self) -> Option<Hash> {
        self.data.prev_block_hash.clone()
    }

    pub fn get_nonce(&self) -> BlockNonce {
        self.data.nonce.clone()
    }

    pub fn get_difficulty_target(&self) -> BlockDifficultyTarget {
        self.data.difficulty_target.clone()
    }

    pub fn get_transactions_merkle_root(&self) -> &TransactionsMerkleRoot {
        &self.data.transactions_merkle_root
    }

    pub fn get_transactions(&self) -> &Vec<NonValidatedTransaction> {
        &self.data.transactions
    }

    pub fn get_timestamp(&self) -> DateTime {
        self.data.timestamp.clone()
    }

    pub fn is_genesis_block(&self) -> bool {
        self.data.is_genesis_block()
    }
}

impl TryEncode for Block {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for Block {
    fn try_decode(data: &[u8]) -> Result<Self, AppError> {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}
