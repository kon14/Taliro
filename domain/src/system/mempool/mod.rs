#[cfg(test)]
mod tests;

use crate::entities::block::Block;
use crate::entities::transaction::Transaction;
use crate::types::hash::Hash;
use async_trait::async_trait;
use common::error::AppError;
use common::log_mempool_info;
use common::params::PaginationParams;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[async_trait]
pub(crate) trait Mempool: Send + Sync + std::fmt::Debug {
    /// Updates the mempool by removing transactions that are included in the given block.
    async fn apply_block(&self, block: &Block) -> Result<(), AppError>;

    /// Adds a transaction to the mempool.<br />
    /// Inserting a transaction with a conflicting hash will overwrite the existing one.
    async fn add_transaction(&self, tx: Transaction) -> Result<(), AppError>;

    /// Retrieves a transaction from the mempool by its hash.
    async fn get_transaction(&self, tx_hash: &Hash) -> Option<Transaction>;

    /// Retrieves a paginated transactions from the mempool.<br />
    /// Returns a tuple containing the list of transactions and the total count of transactions in the mempool.
    async fn get_paginated_transactions(
        &self,
        pagination: PaginationParams,
    ) -> Result<(Vec<Transaction>, usize), AppError>;
}

#[derive(Debug)]
pub(crate) struct DefaultMempool {
    transactions: RwLock<HashMap<Hash, Transaction>>,
}

#[async_trait]
impl Mempool for DefaultMempool {
    async fn apply_block(&self, block: &Block) -> Result<(), AppError> {
        log_mempool_info!("Mempool.apply_block() | block ({:?}) ", &block);

        for tx in block.get_transactions() {
            let tx_hash = tx.get_hash();
            self.rm_transaction(&tx_hash).await;
        }

        log_mempool_info!(
            "Mempool.apply_block(): Transactions successfully updated for block ({}) ",
            block.get_hash()
        );
        Ok(())
    }

    async fn add_transaction(&self, tx: Transaction) -> Result<(), AppError> {
        let tx_hash = tx.get_hash();
        let mut txs = self.transactions.write().await;
        txs.insert(tx_hash, tx);
        Ok(())
    }

    async fn get_transaction(&self, tx_hash: &Hash) -> Option<Transaction> {
        let txs = self.transactions.read().await;
        txs.get(tx_hash).cloned()
    }

    async fn get_paginated_transactions(
        &self,
        pagination: PaginationParams,
    ) -> Result<(Vec<Transaction>, usize), AppError> {
        let txs = self.transactions.read().await;

        let count = txs.len();
        let transactions = txs
            .values()
            .skip(pagination.skip)
            .take(pagination.limit)
            .cloned()
            .collect();
        Ok((transactions, count))
    }
}

impl DefaultMempool {
    pub(crate) fn new() -> Self {
        Self {
            transactions: RwLock::new(HashMap::new()),
        }
    }

    async fn rm_transaction(&self, tx_hash: &Hash) -> Option<Transaction> {
        let mut txs = self.transactions.write().await;
        txs.remove(tx_hash)
    }
}
