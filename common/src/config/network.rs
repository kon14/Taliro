use crate::error::AppError;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PartialNetworkConfig {
    pub listen_address: Option<String>,
    pub init_peers: Option<Vec<String>>,
    pub identity_key_pair: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NetworkConfig {
    pub listen_address: String,            // NetworkAddress
    pub init_peers: Vec<String>,           // HashSet<NetworkAddress>
    pub identity_key_pair: Option<String>, // Base64(ed25519::SigningKey)
}

impl NetworkConfig {
    // [WARNING] Ephemeral port (tcp/0) used by default; peers won't auto-reconnect on restart.
    const DEFAULT_LISTEN_ADDRESS: &'static str = "/ip4/0.0.0.0/tcp/0";
    const DEFAULT_INIT_PEER_ADDRESSES: &'static [&'static str] = &[];

    pub(super) fn from_parts(
        base: PartialNetworkConfig,
        overrides: PartialNetworkConfig,
    ) -> Result<Self, AppError> {
        let listen_address = overrides
            .listen_address
            .or(base.listen_address)
            .unwrap_or(Self::DEFAULT_LISTEN_ADDRESS.to_string());

        let init_peers = overrides.init_peers.or(base.init_peers).unwrap_or(
            Self::DEFAULT_INIT_PEER_ADDRESSES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );

        let identity_key_pair = overrides.identity_key_pair.or(base.identity_key_pair);

        let config = NetworkConfig {
            listen_address,
            init_peers,
            identity_key_pair,
        };
        Ok(config)
    }
}
