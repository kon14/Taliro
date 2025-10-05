use crate::system::blockchain::Blockchain;
use crate::system::mempool::Mempool;
use crate::system::network::P2PNetworkHandle;
use crate::system::node::bus::CommandReceiver;
use crate::system::node::state::boot::NodeBootstrapped;
use crate::system::node::state::run::NodeRunning;
use crate::system::queue::{BlockProcessingQueue, BlockSyncQueue};
use crate::system::utxo::{UtxoSetReader, UtxoSetWriter};
use crate::system::validation::block::BlockValidator;
use crate::system::validation::transaction::TransactionValidator;
use common::config::node::NodeConfig;
use common::error::AppError;
use common::log_node_info;
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeStarted {
    pub(super) cfg: NodeConfig,
    pub(super) blockchain: Arc<dyn Blockchain>,
    pub(super) mempool: Arc<dyn Mempool>,
    pub(super) utxo_set_rw: (Arc<dyn UtxoSetReader>, Arc<dyn UtxoSetWriter>),
    pub(super) network: Arc<dyn P2PNetworkHandle>,
    pub(super) block_sync_queue: Arc<dyn BlockSyncQueue>,
    pub(super) block_proc_queue: Arc<dyn BlockProcessingQueue>,
    pub(super) block_validator: Arc<dyn BlockValidator>,
    pub(super) tx_validator: Arc<dyn TransactionValidator>,
}

impl NodeStarted {
    pub(super) fn start(node: NodeBootstrapped) -> Result<Self, AppError> {
        log_node_info!("Node is starting...");

        let node = Self {
            cfg: node.cfg,
            blockchain: node.blockchain,
            mempool: node.mempool,
            utxo_set_rw: node.utxo_set_rw,
            network: node.network,
            block_sync_queue: node.block_sync_queue,
            block_proc_queue: node.block_proc_queue,
            block_validator: node.block_validator,
            tx_validator: node.tx_validator,
        };

        log_node_info!("Node started successfully.");
        Ok(node)
    }

    pub async fn run(
        self,
        bus_rx: Box<dyn CommandReceiver>,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), AppError> {
        let node = NodeRunning::new(self).await?;
        node.run(bus_rx, shutdown_tx, shutdown_rx).await
    }
}
