use crate::entities::transaction::{TransactionOutPoint, TransactionOutput, Utxo};
use common::error::AppError;
use common::tx::UnitOfWork;
use common::tx::ctx::AtomicTransactionContext;
use std::fmt::Debug;
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
pub trait UtxoRepository: Send + Sync + Debug {
    fn get_utxo_set_append_block_unit_of_work(&self) -> Arc<dyn UnitOfWork>;

    fn get_output<'a>(
        &self,
        tx_ctx: Option<&'a dyn AtomicTransactionContext>,
        outpoint: &TransactionOutPoint,
    ) -> Result<Option<TransactionOutput>, AppError>;

    // TODO: pagination
    fn get_multiple_utxos(&self) -> Result<Vec<Utxo>, AppError>;

    fn insert_utxo<'a>(
        &self,
        tx_ctx: Option<&'a dyn AtomicTransactionContext>,
        utxo: Utxo,
    ) -> Result<(), AppError>;

    fn delete_utxo<'a>(
        &self,
        tx_ctx: Option<&'a dyn AtomicTransactionContext>,
        outpoint: &TransactionOutPoint,
    ) -> Result<(), AppError>;

    fn get_utxo_count(&self) -> usize;
}
