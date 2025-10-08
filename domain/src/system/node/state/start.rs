use crate::system::blockchain::Blockchain;
use crate::system::mempool::Mempool;
use crate::system::network::P2PNetworkHandle;
use crate::system::node::cmd::CommandReceiver;
use crate::system::node::cmd::handlers::{
    BlockchainCommandHandler, CommandDispatcher, MempoolCommandHandler, NetworkCommandHandler,
    UtxoCommandHandler,
};
use crate::system::node::state::boot::NodeBootstrapped;
use crate::system::node::state::run::NodeRunning;
use crate::system::queue::BlockProcessingQueue;
use crate::system::utxo::{UtxoSetReader, UtxoSetWriter};
use crate::system::validation::block::BlockValidator;
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
    pub(super) block_proc_queue: Arc<dyn BlockProcessingQueue>,
    pub(super) block_validator: Arc<dyn BlockValidator>,
    pub(super) cmd_dispatcher: CommandDispatcher,
}

impl NodeStarted {
    pub(super) fn start(node: NodeBootstrapped) -> Result<Self, AppError> {
        log_node_info!("Node is starting...");

        // Build command handlers
        let blockchain_handler = BlockchainCommandHandler::new(
            node.blockchain.clone(),
            node.block_validator.clone(),
            node.utxo_set_rw.1.clone(),
            node.mempool.clone(),
            node.network.clone(),
        );
        let mempool_handler = MempoolCommandHandler::new(node.mempool.clone(), node.tx_validator);
        let network_handler = NetworkCommandHandler::new(
            node.network.clone(),
            node.blockchain.clone(),
            node.block_sync_queue.clone(),
        );
        let utxo_handler = UtxoCommandHandler::new(node.utxo_set_rw.0.clone());
        let cmd_dispatcher = CommandDispatcher::new(
            blockchain_handler,
            mempool_handler,
            network_handler,
            utxo_handler,
        );

        let node = Self {
            cfg: node.cfg,
            blockchain: node.blockchain,
            mempool: node.mempool,
            utxo_set_rw: node.utxo_set_rw,
            network: node.network,
            block_proc_queue: node.block_proc_queue,
            block_validator: node.block_validator,
            cmd_dispatcher,
        };

        log_node_info!("Node started successfully.");
        Ok(node)
    }

    pub async fn run(
        self,
        cmd_rx: Box<dyn CommandReceiver>,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), AppError> {
        let node = NodeRunning::new(self);
        node.run(cmd_rx, shutdown_tx, shutdown_rx).await
    }
}
