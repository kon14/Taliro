use common::error::AppError;
use dotenv::dotenv;
use std::env;

pub(crate) struct EnvironmentConfig {
    // HTTP
    pub(crate) http_api_port: Option<u16>,
    pub(crate) http_api_base_url: Option<String>,
    pub(crate) http_master_key_secret: Option<String>,
    // Network
    pub(crate) network_listen_address: Option<String>,
    pub(crate) network_init_peers: Option<Vec<String>>,
    pub(crate) network_identity_key_pair: Option<String>,
    // Storage
    pub(crate) db_path: Option<String>,
}

pub(crate) fn setup_env() {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
}

impl EnvironmentConfig {
    const HTTP_API_PORT_ENV: &'static str = "HTTP_API_PORT";
    const HTTP_API_BASE_URL_ENV: &'static str = "HTTP_API_BASE_URL";
    const HTTP_MASTER_KEY_SECRET_ENV: &'static str = "HTTP_MASTER_KEY_SECRET";
    const NETWORK_LISTEN_ADDRESS_ENV: &'static str = "NETWORK_LISTEN_ADDRESS";
    const NETWORK_INIT_PEERS_ENV: &'static str = "NETWORK_INIT_PEERS";
    const NETWORK_IDENTITY_KEY_PAIR_ENV: &'static str = "NETWORK_IDENTITY_KEY_PAIR";
    const STORAGE_DB_PATH_ENV: &'static str = "STORAGE_DB_PATH";

    pub(crate) fn load() -> Result<Self, AppError> {
        Ok(Self {
            http_api_port: Self::get_http_api_port(),
            http_api_base_url: Self::get_http_api_base_url(),
            http_master_key_secret: Self::get_http_master_key_secret(),
            network_listen_address: Self::get_network_listen_address(),
            network_init_peers: Self::get_network_init_peers(),
            network_identity_key_pair: Self::get_network_identity_key_pair(),
            db_path: Self::get_storage_db_path(),
        })
    }

    fn get_http_api_port() -> Option<u16> {
        match env::var(Self::HTTP_API_PORT_ENV) {
            Ok(env_str) => env_str.parse::<u16>().ok(),
            Err(_) => None,
        }
    }

    fn get_http_api_base_url() -> Option<String> {
        match env::var(Self::HTTP_API_BASE_URL_ENV) {
            Ok(env_str) if !env_str.trim().is_empty() => Some(env_str),
            _ => None,
        }
    }

    fn get_http_master_key_secret() -> Option<String> {
        match env::var(Self::HTTP_MASTER_KEY_SECRET_ENV) {
            Ok(env_str) if !env_str.trim().is_empty() => Some(env_str),
            _ => None,
        }
    }

    fn get_network_listen_address() -> Option<String> {
        match env::var(Self::NETWORK_LISTEN_ADDRESS_ENV) {
            Ok(env_str) if !env_str.trim().is_empty() => Some(env_str),
            _ => None,
        }
    }

    fn get_network_init_peers() -> Option<Vec<String>> {
        let peer_addr_str = match env::var(Self::NETWORK_INIT_PEERS_ENV) {
            Ok(env_str) if !env_str.trim().is_empty() => Some(env_str),
            _ => None,
        }?;

        let peer_addresses: Vec<String> = peer_addr_str
            .split(';')
            .map(str::trim)
            .filter(|addr_str| !(*addr_str).is_empty())
            .map(String::from)
            .collect();
        if peer_addresses.is_empty() {
            None
        } else {
            Some(peer_addresses)
        }
    }

    fn get_network_identity_key_pair() -> Option<String> {
        match env::var(Self::NETWORK_IDENTITY_KEY_PAIR_ENV) {
            Ok(env_str) if !env_str.trim().is_empty() => Some(env_str),
            _ => None,
        }
    }

    fn get_storage_db_path() -> Option<String> {
        match env::var(Self::STORAGE_DB_PATH_ENV) {
            Ok(env_str) if !env_str.trim().is_empty() => Some(env_str),
            _ => None,
        }
    }
}
