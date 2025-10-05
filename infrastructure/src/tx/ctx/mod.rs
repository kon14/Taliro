use common::error::AppError;
use common::tx::ctx::AtomicTransactionContext;
use sled::transaction::TransactionalTree;
use std::any::TypeId;

#[derive(Clone)]
pub(super) struct SledTxBlockchainAppendBlockContext {
    pub(super) blocks_tree: TransactionalTree,
    pub(super) heights_tree: TransactionalTree,
    pub(super) outbox_unprocessed_tree: TransactionalTree,
}

impl AtomicTransactionContext for SledTxBlockchainAppendBlockContext {
    fn type_id(&self) -> TypeId {
        TypeId::of::<SledTxBlockchainAppendBlockContext>()
    }

    fn as_any(&self) -> Box<dyn std::any::Any> {
        let ctx = SledTxBlockchainAppendBlockContext {
            blocks_tree: self.blocks_tree.clone(),
            heights_tree: self.heights_tree.clone(),
            outbox_unprocessed_tree: self.outbox_unprocessed_tree.clone(),
        };
        Box::new(ctx)
    }
}

#[derive(Clone)]
pub(super) struct SledTxUtxoSetAppendBlockContext {
    pub(super) utxo_tree: TransactionalTree,
}

impl AtomicTransactionContext for SledTxUtxoSetAppendBlockContext {
    fn type_id(&self) -> TypeId {
        TypeId::of::<SledTxUtxoSetAppendBlockContext>()
    }

    fn as_any(&self) -> Box<dyn std::any::Any> {
        let ctx = SledTxUtxoSetAppendBlockContext {
            utxo_tree: self.utxo_tree.clone(),
        };
        Box::new(ctx)
    }
}

impl crate::ext::TransactionContextExtInfrastructure for &dyn AtomicTransactionContext {
    fn get_blocks_tree(&self) -> Result<TransactionalTree, AppError> {
        let tree = match self.type_id() {
            type_id if type_id == TypeId::of::<SledTxBlockchainAppendBlockContext>() => self
                .as_any()
                .downcast_ref::<SledTxBlockchainAppendBlockContext>()
                .ok_or_else(|| {
                    AppError::internal(
                        "Mismatched transaction context type id. Couldn't downcast type!",
                    )
                })?
                .blocks_tree
                .clone(),
            _ => Err(AppError::internal("Invalid transaction context type!"))?,
        };
        Ok(tree)
    }

    fn get_heights_tree(&self) -> Result<TransactionalTree, AppError> {
        let tree = match self.type_id() {
            type_id if type_id == TypeId::of::<SledTxBlockchainAppendBlockContext>() => self
                .as_any()
                .downcast_ref::<SledTxBlockchainAppendBlockContext>()
                .ok_or_else(|| {
                    AppError::internal(
                        "Mismatched transaction context type id. Couldn't downcast type!",
                    )
                })?
                .heights_tree
                .clone(),
            _ => Err(AppError::internal("Invalid transaction context type!"))?,
        };
        Ok(tree)
    }

    fn get_meta_tree(&self) -> Result<TransactionalTree, AppError> {
        let tree = match self.type_id() {
            _ => Err(AppError::internal("Invalid transaction context type!"))?,
        };
        Ok(tree)
    }

    fn get_utxo_tree(&self) -> Result<TransactionalTree, AppError> {
        let tree = match self.type_id() {
            type_id if type_id == TypeId::of::<SledTxUtxoSetAppendBlockContext>() => self
                .as_any()
                .downcast_ref::<SledTxUtxoSetAppendBlockContext>()
                .ok_or_else(|| {
                    AppError::internal(
                        "Mismatched transaction context type id. Couldn't downcast type!",
                    )
                })?
                .utxo_tree
                .clone(),
            _ => Err(AppError::internal("Invalid transaction context type!"))?,
        };
        Ok(tree)
    }

    fn get_outbox_unprocessed_tree(&self) -> Result<TransactionalTree, AppError> {
        let tree = match self.type_id() {
            type_id if type_id == TypeId::of::<SledTxBlockchainAppendBlockContext>() => self
                .as_any()
                .downcast_ref::<SledTxBlockchainAppendBlockContext>()
                .ok_or_else(|| {
                    AppError::internal(
                        "Mismatched transaction context type id. Couldn't downcast type!",
                    )
                })?
                .outbox_unprocessed_tree
                .clone(),
            _ => Err(AppError::internal("Invalid transaction context type!"))?,
        };
        Ok(tree)
    }
}
