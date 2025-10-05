pub(crate) mod ctx;
pub(crate) mod trees;

use crate::ext::AppErrorExtInfrastructure;
use crate::tx::ctx::{SledTxBlockchainAppendBlockContext, SledTxUtxoSetAppendBlockContext};
use crate::tx::trees::SledTxTrees;
use common::{
    error::AppError,
    tx::{AtomicTransactionOutput, UnitOfWork, ctx::AtomicTransactionContext},
};
use sled::Transactional;
use sled::transaction::{ConflictableTransactionError, TransactionError};
use std::cell::RefCell;

pub(crate) struct SledUnitOfWork {
    trees: SledTxTrees,
}

impl SledUnitOfWork {
    pub(crate) fn new(trees: SledTxTrees) -> Self {
        Self { trees }
    }
}

impl UnitOfWork for SledUnitOfWork {
    fn run_in_transaction(
        &self,
        f: Box<
            dyn for<'a> FnMut(
                    &'a mut dyn AtomicTransactionContext,
                ) -> Result<AtomicTransactionOutput, AppError>
                + Send,
        >,
    ) -> Result<AtomicTransactionOutput, AppError> {
        let f = RefCell::new(f);

        let tx_res: Result<_, TransactionError<AppError>> = match self.trees.clone() {
            SledTxTrees::BlockchainAppendBlock(trees) => (
                &trees.blocks_tree,
                &trees.heights_tree,
                &trees.outbox_unprocessed_tree,
            )
                .transaction(|(blocks_tree, heights_tree, outbox_unprocessed_tree)| {
                    let mut ctx = SledTxBlockchainAppendBlockContext {
                        blocks_tree: blocks_tree.clone(),
                        heights_tree: heights_tree.clone(),
                        outbox_unprocessed_tree: outbox_unprocessed_tree.clone(),
                    };
                    f.borrow_mut()(&mut ctx)
                        .map_err(ConflictableTransactionError::Abort::<AppError>)
                }),
            SledTxTrees::UtxoSetAppendBlock(trees) => trees.utxo_tree.transaction(|utxo_tree| {
                let mut ctx = SledTxUtxoSetAppendBlockContext {
                    utxo_tree: utxo_tree.clone(),
                };
                f.borrow_mut()(&mut ctx).map_err(ConflictableTransactionError::Abort::<AppError>)
            }),
        };
        tx_res.to_app_error()
    }
}
