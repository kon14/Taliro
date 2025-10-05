use common::config::network::NetworkConfig;
use common::error::AppError;
use domain::repos::network::NetworkRepository;
use domain::system::network::P2PNetworkEngine;
use infrastructure::network::engine::Libp2pNetworkEngine;
use std::sync::Arc;

pub(crate) fn build_p2p_network(
    cfg: NetworkConfig,
    network_repo: Arc<dyn NetworkRepository>,
) -> Result<Box<dyn P2PNetworkEngine>, AppError> {
    let network = Libp2pNetworkEngine::new(cfg, network_repo)?;
    Ok(Box::new(network))
}
