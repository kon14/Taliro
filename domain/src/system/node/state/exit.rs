use crate::system::blockchain::Blockchain;
use crate::system::mempool::Mempool;
use crate::system::network::P2PNetworkHandle;
use crate::system::node::state::run::NodeRunning;
use crate::system::utxo::{UtxoSetReader, UtxoSetWriter};
use common::config::node::NodeConfig;
use common::error::AppError;
use common::{log_node_error, log_node_info};
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeTerminating {
    #[allow(unused)]
    pub(super) cfg: NodeConfig,
    #[allow(unused)]
    pub(super) blockchain: Arc<dyn Blockchain>,
    #[allow(unused)]
    pub(super) mempool: Arc<dyn Mempool>,
    #[allow(unused)]
    pub(super) utxo_set_rw: (Arc<dyn UtxoSetReader>, Arc<dyn UtxoSetWriter>),
    #[allow(unused)]
    pub(super) network: Arc<dyn P2PNetworkHandle>,
}

impl NodeTerminating {
    pub(super) fn terminate(
        node: NodeRunning,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
    ) -> Result<Self, AppError> {
        log_node_info!("Node is terminating...");

        let node = Self {
            cfg: node.cfg,
            blockchain: node.blockchain,
            mempool: node.mempool,
            utxo_set_rw: (node.utxo_set_r, node.utxo_set_w),
            network: node.network,
        };

        node.handle_termination(shutdown_tx);

        log_node_info!("Node terminated.");
        Ok(node)
    }

    fn handle_termination(&self, shutdown_tx: tokio::sync::broadcast::Sender<()>) {
        if let Err(_) = shutdown_tx.send(()) {
            log_node_error!("Failed to broadcast node termination signal!");
        };
    }
}
