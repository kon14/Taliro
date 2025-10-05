use crate::entities::block::BlockHeight;
use crate::repos::blockchain::BlockchainRepository;
use crate::repos::outbox::OutboxRepository;
use crate::repos::utxo::UtxoRepository;
use crate::system::blockchain::{Blockchain, DefaultBlockchain};
use crate::system::mempool::{DefaultMempool, Mempool};
use crate::system::network::P2PNetworkEngine;
use crate::system::node::bus::{CommandResponderFactory, CommandSender};
use crate::system::node::state::boot::NodeBootstrapped;
use crate::system::queue::{BlockProcessingQueue, BlockSyncQueue};
use crate::system::utxo::{UtxoReaderService, UtxoSetReader, UtxoSetWriter, UtxoSetWriterService};
use crate::system::validation::block::{BlockValidator, DefaultBlockValidator};
use crate::system::validation::transaction::{DefaultTransactionValidator, TransactionValidator};
use crate::types::hash::Hash;
use common::config::node::NodeConfig;
use common::error::AppError;
use common::log_node_info;
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeInitialized {
    pub(super) cfg: NodeConfig,
    pub(super) blockchain: Arc<dyn Blockchain>,
    pub(super) mempool: Arc<dyn Mempool>,
    pub(super) utxo_set_rw: (Arc<dyn UtxoSetReader>, Arc<dyn UtxoSetWriter>),
    pub(super) network: Box<dyn P2PNetworkEngine>,
    pub(super) block_validator: Arc<dyn BlockValidator>,
    pub(super) tx_validator: Arc<dyn TransactionValidator>,
}

impl NodeInitialized {
    pub(in super::super) async fn init(
        cfg: NodeConfig,
        blockchain_repo: Arc<dyn BlockchainRepository>,
        utxo_repo: Arc<dyn UtxoRepository>,
        outbox_repo: Arc<dyn OutboxRepository>,
        network: Box<dyn P2PNetworkEngine>,
    ) -> Result<Self, AppError> {
        log_node_info!("Node initializing...");

        let utxo_set_r = Arc::new(UtxoReaderService::new(utxo_repo.clone()));
        let utxo_set_w = Arc::new(UtxoSetWriterService::new(utxo_repo));
        let blockchain = Arc::new(DefaultBlockchain::new(blockchain_repo, outbox_repo.clone()));
        let mempool = Arc::new(DefaultMempool::new());
        let tx_validator = Arc::new(DefaultTransactionValidator::new(utxo_set_r.clone()));
        let block_validator = Arc::new(DefaultBlockValidator::new(
            blockchain.clone(),
            tx_validator.clone(),
        ));

        let node = Self {
            cfg,
            blockchain,
            mempool,
            utxo_set_rw: (utxo_set_r, utxo_set_w),
            network,
            block_validator,
            tx_validator,
        };

        log_node_info!("Node initialized successfully.");
        Ok(node)
    }

    pub async fn get_tip_info(&self) -> Result<Option<(Hash, BlockHeight)>, AppError> {
        self.blockchain.get_tip_info().await
    }

    pub async fn bootstrap(
        self,
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
        block_sync_queue: Arc<dyn BlockSyncQueue>,
        block_proc_queue: Arc<dyn BlockProcessingQueue>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<NodeBootstrapped, AppError> {
        NodeBootstrapped::bootstrap(
            self,
            bus_tx,
            bus_tx_res_factory,
            block_sync_queue,
            block_proc_queue,
            shutdown_rx,
        )
        .await
    }
}
