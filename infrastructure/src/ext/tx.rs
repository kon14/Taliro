use common::error::AppError;
use sled::transaction::TransactionalTree;

pub(crate) trait TransactionContextExtInfrastructure {
    fn get_blocks_tree(&self) -> Result<TransactionalTree, AppError>;
    fn get_heights_tree(&self) -> Result<TransactionalTree, AppError>;
    fn get_meta_tree(&self) -> Result<TransactionalTree, AppError>;
    fn get_utxo_tree(&self) -> Result<TransactionalTree, AppError>;
    fn get_outbox_unprocessed_tree(&self) -> Result<TransactionalTree, AppError>;
}
