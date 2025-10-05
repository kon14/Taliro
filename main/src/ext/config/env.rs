use super::PartialAppConfigFromEnvExtMain;
use crate::bootstrap::env::EnvironmentConfig;
use common::config::http::PartialHttpConfig;
use common::config::network::PartialNetworkConfig;
use common::config::node::PartialNodeConfig;
use common::config::storage::PartialStorageConfig;
use common::config::PartialAppConfig;
use common::error::AppError;

impl PartialAppConfigFromEnvExtMain for PartialAppConfig {
    fn load_from_env() -> Result<Self, AppError> {
        // Note: This could use a refactor...
        let env = EnvironmentConfig::load()?;
        let config = Self {
            http: PartialHttpConfig {
                api_port: env.http_api_port,
                api_base_url: env.http_api_base_url,
                master_key_secret: env.http_master_key_secret,
            },
            network: PartialNetworkConfig {
                listen_address: env.network_listen_address,
                init_peers: env.network_init_peers,
                identity_key_pair: env.network_identity_key_pair,
            },
            node: PartialNodeConfig {},
            storage: PartialStorageConfig {
                db_path: env.db_path,
            },
        };
        Ok(config)
    }
}
