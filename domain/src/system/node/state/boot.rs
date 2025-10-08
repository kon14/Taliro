use crate::system::blockchain::Blockchain;
use crate::system::mempool::Mempool;
use crate::system::network::P2PNetworkHandle;
use crate::system::node::cmd::{CommandResponderFactory, CommandSender};
use crate::system::node::state::init::NodeInitialized;
use crate::system::node::state::start::NodeStarted;
use crate::system::queue::{BlockProcessingQueue, BlockSyncQueue};
use crate::system::utxo::{UtxoSetReader, UtxoSetWriter};
use crate::system::validation::block::BlockValidator;
use crate::system::validation::transaction::TransactionValidator;
use common::config::node::NodeConfig;
use common::error::AppError;
use common::log_node_info;
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeBootstrapped {
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

impl NodeBootstrapped {
    pub(super) async fn bootstrap(
        node: NodeInitialized,
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
        block_sync_queue: Arc<dyn BlockSyncQueue>,
        block_proc_queue: Arc<dyn BlockProcessingQueue>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Self, AppError> {
        log_node_info!("Node is bootstrapping...");

        // Connect to P2P network
        let network = node
            .network
            .connect(cmd_tx, cmd_tx_res_factory, shutdown_rx)
            .await?;

        let node = Self {
            cfg: node.cfg,
            blockchain: node.blockchain,
            mempool: node.mempool,
            utxo_set_rw: node.utxo_set_rw,
            network,
            block_sync_queue,
            block_proc_queue,
            block_validator: node.block_validator,
            tx_validator: node.tx_validator,
        };

        log_node_info!("Node bootstrapped successfully.");
        Ok(node)
    }

    pub fn start(self) -> Result<NodeStarted, AppError> {
        NodeStarted::start(self)
    }
}
