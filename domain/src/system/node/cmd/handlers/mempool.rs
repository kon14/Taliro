use super::super::CommandResponder;
use super::CommandHandlerControlFlow;
use crate::entities::transaction::{NonValidatedTransaction, Transaction};
use crate::system::mempool::Mempool;
use crate::system::validation::transaction::TransactionValidator;
use crate::types::hash::Hash;
use common::error::AppError;
use common::params::PaginationParams;
use common::{log_node_debug, log_node_error};
use std::sync::Arc;

/// Handles mempool-related commands.
#[derive(Debug, Clone)]
pub(crate) struct MempoolCommandHandler {
    mempool: Arc<dyn Mempool>,
    tx_validator: Arc<dyn TransactionValidator>,
}

impl MempoolCommandHandler {
    pub(crate) fn new(
        mempool: Arc<dyn Mempool>,
        tx_validator: Arc<dyn TransactionValidator>,
    ) -> Self {
        Self {
            mempool,
            tx_validator,
        }
    }

    /// Place a transaction in the mempool after validation.
    pub(in crate::system::node) async fn handle_place_transaction(
        &self,
        tx: NonValidatedTransaction,
        responder: Box<dyn CommandResponder<Result<Transaction, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "MempoolCommandHandler: Placing transaction | Hash: {}",
            tx.get_hash()
        );

        let res = async {
            let tx = self.tx_validator.validate_transaction(tx).await?;
            self.mempool.add_transaction(tx.clone()).await?;
            Ok(tx)
        }
        .await;

        if let Err(ref err) = res {
            log_node_error!("Failed to place transaction: {}", err);
        }

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Get paginated transactions from mempool.
    pub(in crate::system::node) async fn handle_get_paginated_transactions(
        &self,
        pagination: PaginationParams,
        responder: Box<dyn CommandResponder<Result<(Vec<Transaction>, usize), AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "MempoolCommandHandler: Getting paginated transactions | Skip: {}, Limit: {}",
            pagination.skip,
            pagination.limit,
        );

        let res = self.mempool.get_paginated_transactions(pagination).await;

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Get specific transactions by their hashes.
    pub(in crate::system::node) async fn handle_get_transactions_by_hashes(
        &self,
        tx_hashes: Vec<Hash>,
        responder: Box<dyn CommandResponder<Result<Vec<Transaction>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "MempoolCommandHandler: Getting {} transaction(s) by hash",
            tx_hashes.len()
        );

        let res = self.get_transactions_by_hashes_internal(tx_hashes).await;

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    async fn get_transactions_by_hashes_internal(
        &self,
        tx_hashes: Vec<Hash>,
    ) -> Result<Vec<Transaction>, AppError> {
        let mut transactions = Vec::with_capacity(tx_hashes.len());
        let mut missing_hashes = Vec::new();

        for tx_hash in &tx_hashes {
            if let Some(tx) = self.mempool.get_transaction(tx_hash).await {
                transactions.push(tx);
            } else {
                missing_hashes.push(tx_hash.clone());
            }
        }

        if !missing_hashes.is_empty() {
            return Err(AppError::bad_request(format!(
                "Transaction count mismatch! Expected {} transaction(s), but only found {}. Missing hashes: {:?}",
                tx_hashes.len(),
                transactions.len(),
                missing_hashes
            )));
        }

        Ok(transactions)
    }
}
