use crate::entities::block::Block;
use crate::entities::transaction::Transaction;
use crate::types::hash::Hash;
use async_trait::async_trait;
use common::error::AppError;
use common::log_mempool_info;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[async_trait]
pub(crate) trait Mempool: Send + Sync + std::fmt::Debug {
    async fn apply_block(&self, block: &Block) -> Result<(), AppError>;
    async fn add_transaction(&self, tx: Transaction) -> Result<(), AppError>;
    async fn get_transaction(&self, tx_hash: &Hash) -> Option<Transaction>;
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
