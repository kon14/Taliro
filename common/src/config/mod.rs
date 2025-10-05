pub mod http;
pub mod network;
pub mod node;
pub mod storage;

use crate::error::AppError;
use serde::Deserialize;

// TODO: Common is currently pulling in serde.
// This is not ideal, but it's a trade-off to avoid code duplication across crates.
// Let's revisit this at some point.

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PartialAppConfig {
    pub http: http::PartialHttpConfig,
    pub network: network::PartialNetworkConfig,
    pub node: node::PartialNodeConfig,
    pub storage: storage::PartialStorageConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub http: http::HttpConfig,
    pub network: network::NetworkConfig,
    pub node: node::NodeConfig,
    pub storage: storage::StorageConfig,
}

impl AppConfig {
    pub fn from_parts(
        base_cfg: PartialAppConfig,
        overrides_cfg: PartialAppConfig,
    ) -> Result<Self, AppError> {
        let http_cfg = http::HttpConfig::from_parts(base_cfg.http, overrides_cfg.http)?;
        let node_cfg = node::NodeConfig::from_parts(base_cfg.node, overrides_cfg.node)?;
        let network_cfg =
            network::NetworkConfig::from_parts(base_cfg.network, overrides_cfg.network)?;
        let storage_cfg =
            storage::StorageConfig::from_parts(base_cfg.storage, overrides_cfg.storage)?;

        Ok(Self {
            http: http_cfg,
            network: network_cfg,
            node: node_cfg,
            storage: storage_cfg,
        })
    }
}
