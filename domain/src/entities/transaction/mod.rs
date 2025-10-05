mod amount;
pub(in super::super::entities) mod inner;
mod io;
mod merkle;
mod outpoint;
mod utxo;

pub use amount::TransactionAmount;
pub use io::{TransactionInput, TransactionOutput};
pub use merkle::TransactionsMerkleRoot;
pub use outpoint::TransactionOutPoint;
pub use utxo::Utxo;

use crate::encode::{TryDecode, TryEncode};
use crate::entities::transaction::inner::InnerTransaction;
use crate::ext::AppErrorConvertibleDomain;
use crate::types::hash::{Hash, TryHashable};
use crate::types::time::DateTime;
use bincode::{Decode, Encode};
use common::error::AppError;

#[derive(Clone, Debug, Encode, Decode)]
pub struct Transaction {
    hash: Hash,
    data: InnerTransaction,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct NonValidatedTransaction {
    hash: Hash,
    data: InnerTransaction,
}

impl Transaction {
    /// Internal method to construct a validated [`Transaction`] from a [`NonValidatedTransaction`].<br />
    /// Called exclusively post-validation.
    pub(crate) fn _new_validated(tx: NonValidatedTransaction) -> Transaction {
        Self {
            hash: tx.hash,
            data: tx.data,
        }
    }
}

impl NonValidatedTransaction {
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        timestamp: DateTime,
    ) -> Result<Self, AppError> {
        let inner_tx = InnerTransaction {
            inputs,
            outputs,
            timestamp,
        };
        let hash = inner_tx.try_hash()?;
        let tx = Self {
            hash,
            data: inner_tx,
        };
        Ok(tx)
    }
}

impl Transaction {
    pub fn get_hash(&self) -> Hash {
        self.hash.clone()
    }

    pub fn get_inputs(&self) -> &Vec<TransactionInput> {
        &self.data.inputs
    }

    pub fn get_outputs(&self) -> &Vec<TransactionOutput> {
        &self.data.outputs
    }

    pub fn get_timestamp(&self) -> DateTime {
        self.data.timestamp.clone()
    }

    #[allow(unused)]
    pub(crate) fn is_coinbase_tx(&self) -> bool {
        self.data.inputs.is_empty()
    }

    pub fn invalidate(self) -> NonValidatedTransaction {
        NonValidatedTransaction {
            hash: self.hash,
            data: self.data,
        }
    }
}

impl NonValidatedTransaction {
    pub fn get_hash(&self) -> Hash {
        self.hash.clone()
    }

    pub fn get_inputs(&self) -> &Vec<TransactionInput> {
        &self.data.inputs
    }

    pub fn get_outputs(&self) -> &Vec<TransactionOutput> {
        &self.data.outputs
    }

    pub fn get_timestamp(&self) -> DateTime {
        self.data.timestamp.clone()
    }

    pub(crate) fn is_coinbase_tx(&self) -> bool {
        self.data.inputs.is_empty()
    }
}

impl TryEncode for Transaction {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for Transaction {
    fn try_decode(data: &[u8]) -> Result<Self, AppError> {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}
