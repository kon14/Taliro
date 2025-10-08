pub mod cmd;
mod state;

pub use state::*;

use crate::repos::blockchain::BlockchainRepository;
use crate::repos::outbox::OutboxRepository;
use crate::repos::utxo::UtxoRepository;
use crate::system::network::P2PNetworkEngine;
use common::config::node::NodeConfig;
use common::error::AppError;
use std::sync::Arc;

pub async fn build_node(
    cfg: NodeConfig,
    blockchain_repo: Arc<dyn BlockchainRepository>,
    utxo_repo: Arc<dyn UtxoRepository>,
    outbox_repo: Arc<dyn OutboxRepository>,
    network: Box<dyn P2PNetworkEngine>,
) -> Result<NodeInitialized, AppError> {
    NodeInitialized::init(cfg, blockchain_repo, utxo_repo, outbox_repo, network).await
}
