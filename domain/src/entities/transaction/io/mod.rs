use crate::encode::{TryDecode, TryEncode};
use crate::entities::transaction::{TransactionAmount, TransactionOutPoint};
use crate::ext::AppErrorConvertibleDomain;
use crate::types::wallet::WalletAddress;
use bincode::{Decode, Encode};
use common::error::AppError;

#[derive(Clone, Debug, Encode, Decode)]
pub struct TransactionInput {
    previous_output: TransactionOutPoint,
    // pub(super) signature: Signature, // TODO
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct TransactionOutput {
    pub(super) recipient: WalletAddress,
    pub(super) amount: TransactionAmount,
}

impl TransactionInput {
    pub fn new(previous_output: TransactionOutPoint) -> Self {
        Self { previous_output }
    }

    pub fn get_previous_output(&self) -> &TransactionOutPoint {
        &self.previous_output
    }
}

impl TransactionOutput {
    pub fn new(recipient: WalletAddress, amount: TransactionAmount) -> Self {
        Self { recipient, amount }
    }

    pub fn get_recipient(&self) -> &WalletAddress {
        &self.recipient
    }

    pub fn get_amount(&self) -> TransactionAmount {
        self.amount.clone()
    }
}

impl TryEncode for TransactionOutput {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for TransactionOutput {
    fn try_decode(data: &[u8]) -> Result<Self, AppError> {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}
