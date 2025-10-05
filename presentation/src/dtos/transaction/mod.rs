use crate::dtos::time::DateTimeExtPresentation;
use chrono::{DateTime, Utc};
use common::error::AppError;
use domain::entities::transaction::{
    Transaction, TransactionInput, TransactionOutPoint, TransactionOutput, Utxo,
};
use domain::types::hash::Hash;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "Transaction")]
pub(crate) struct TransactionPresentationDto {
    hash: String,
    data: InnerTransactionPresentationDto,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "TransactionData")]
struct InnerTransactionPresentationDto {
    inputs: Vec<TransactionInputPresentationDto>,
    outputs: Vec<TransactionOutputPresentationDto>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "TransactionInput")]
struct TransactionInputPresentationDto {
    previous_output: TransactionOutPointPresentationDto,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "TransactionOutput")]
struct TransactionOutputPresentationDto {
    recipient: String,
    amount: u128,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "TransactionOutPoint")]
pub(crate) struct TransactionOutPointPresentationDto {
    tx_id: String,
    tx_output_index: usize,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "Utxo")]
pub(crate) struct UtxoPresentationDto {
    outpoint: TransactionOutPointPresentationDto,
    output: TransactionOutputPresentationDto,
}

impl From<Transaction> for TransactionPresentationDto {
    fn from(tx: Transaction) -> Self {
        Self {
            hash: tx.get_hash().to_string(),
            data: InnerTransactionPresentationDto {
                inputs: tx
                    .get_inputs()
                    .iter()
                    .cloned()
                    .map(|input| input.into())
                    .collect(),
                outputs: tx
                    .get_outputs()
                    .iter()
                    .cloned()
                    .map(|output| output.into())
                    .collect(),
                timestamp: tx.get_timestamp().to_chrono(),
            },
        }
    }
}

impl From<TransactionInput> for TransactionInputPresentationDto {
    fn from(input: TransactionInput) -> Self {
        Self {
            previous_output: input.get_previous_output().clone().into(),
        }
    }
}

impl From<TransactionOutput> for TransactionOutputPresentationDto {
    fn from(output: TransactionOutput) -> Self {
        Self {
            recipient: output.get_recipient().to_string(),
            amount: output.get_amount().as_u128(),
        }
    }
}

impl From<TransactionOutPoint> for TransactionOutPointPresentationDto {
    fn from(outpoint: TransactionOutPoint) -> Self {
        Self {
            tx_id: outpoint.get_tx_id().to_string(),
            tx_output_index: outpoint.get_tx_output_index(),
        }
    }
}

impl From<Utxo> for UtxoPresentationDto {
    fn from(utxo: Utxo) -> Self {
        Self {
            outpoint: utxo.get_outpoint().clone().into(),
            output: utxo.get_output().clone().into(),
        }
    }
}

impl TryFrom<TransactionOutPointPresentationDto> for TransactionOutPoint {
    type Error = AppError;

    fn try_from(dto: TransactionOutPointPresentationDto) -> Result<Self, Self::Error> {
        let tx_id = Hash::try_from(dto.tx_id.as_ref())?;
        let tx_output_index = dto.tx_output_index;
        Ok(Self::new(tx_id, tx_output_index))
    }
}
