use macros::define_log_macros;

pub use ::log as base_log;

pub mod log_targets {
    pub const NODE: &str = "node";
    pub const BLOCKCHAIN: &str = "blockchain";
    pub const MEMPOOL: &str = "mempool";
    pub const UTXO: &str = "utxo";
    pub const NETWORK: &str = "network";
    pub const NETWORK_GOSSIPSUB: &str = "network.gossipsub";
    pub const NETWORK_KADEMLIA: &str = "network.kademlia";
    pub const NETWORK_TALIRO: &str = "network.taliro";
    pub const STORAGE: &str = "storage";
    pub const APPLICATION: &str = "application";
    pub const AUTHENTICATION: &str = "authentication";
    pub const HTTP: &str = "http";
}

define_log_macros![
    common::log::base_log,
    // Node
    (log_node_trace, trace, common::log::log_targets::NODE),
    (log_node_debug, debug, common::log::log_targets::NODE),
    (log_node_info, info, common::log::log_targets::NODE),
    (log_node_warn, warn, common::log::log_targets::NODE),
    (log_node_error, error, common::log::log_targets::NODE),
    // Blockchain
    (log_blk_trace, trace, common::log::log_targets::BLOCKCHAIN),
    (log_blk_debug, debug, common::log::log_targets::BLOCKCHAIN),
    (log_blk_info, info, common::log::log_targets::BLOCKCHAIN),
    (log_blk_warn, warn, common::log::log_targets::BLOCKCHAIN),
    (log_blk_error, error, common::log::log_targets::BLOCKCHAIN),
    // Mempool
    (log_mempool_trace, trace, common::log::log_targets::MEMPOOL),
    (log_mempool_debug, debug, common::log::log_targets::MEMPOOL),
    (log_mempool_info, info, common::log::log_targets::MEMPOOL),
    (log_mempool_warn, warn, common::log::log_targets::MEMPOOL),
    (log_mempool_error, error, common::log::log_targets::MEMPOOL),
    // UTXO
    (log_utxo_trace, trace, common::log::log_targets::UTXO),
    (log_utxo_debug, debug, common::log::log_targets::UTXO),
    (log_utxo_info, info, common::log::log_targets::UTXO),
    (log_utxo_warn, warn, common::log::log_targets::UTXO),
    (log_utxo_error, error, common::log::log_targets::UTXO),
    // Network
    (log_net_trace, trace, common::log::log_targets::NETWORK),
    (log_net_debug, debug, common::log::log_targets::NETWORK),
    (log_net_info, info, common::log::log_targets::NETWORK),
    (log_net_warn, warn, common::log::log_targets::NETWORK),
    (log_net_error, error, common::log::log_targets::NETWORK),
    // Network Gossipsub
    (
        log_net_gs_trace,
        trace,
        common::log::log_targets::NETWORK_GOSSIPSUB
    ),
    (
        log_net_gs_debug,
        debug,
        common::log::log_targets::NETWORK_GOSSIPSUB
    ),
    (
        log_net_gs_info,
        info,
        common::log::log_targets::NETWORK_GOSSIPSUB
    ),
    (
        log_net_gs_warn,
        warn,
        common::log::log_targets::NETWORK_GOSSIPSUB
    ),
    (
        log_net_gs_error,
        error,
        common::log::log_targets::NETWORK_GOSSIPSUB
    ),
    // Network Kademlia
    (
        log_net_kad_trace,
        trace,
        common::log::log_targets::NETWORK_KADEMLIA
    ),
    (
        log_net_kad_debug,
        debug,
        common::log::log_targets::NETWORK_KADEMLIA
    ),
    (
        log_net_kad_info,
        info,
        common::log::log_targets::NETWORK_KADEMLIA
    ),
    (
        log_net_kad_warn,
        warn,
        common::log::log_targets::NETWORK_KADEMLIA
    ),
    (
        log_net_kad_error,
        error,
        common::log::log_targets::NETWORK_KADEMLIA
    ),
    // Network Taliro
    (
        log_net_taliro_trace,
        trace,
        common::log::log_targets::NETWORK_TALIRO
    ),
    (
        log_net_taliro_debug,
        debug,
        common::log::log_targets::NETWORK_TALIRO
    ),
    (
        log_net_taliro_info,
        info,
        common::log::log_targets::NETWORK_TALIRO
    ),
    (
        log_net_taliro_warn,
        warn,
        common::log::log_targets::NETWORK_TALIRO
    ),
    (
        log_net_taliro_error,
        error,
        common::log::log_targets::NETWORK_TALIRO
    ),
    // Storage
    (log_storage_trace, trace, common::log::log_targets::STORAGE),
    (log_storage_debug, debug, common::log::log_targets::STORAGE),
    (log_storage_info, info, common::log::log_targets::STORAGE),
    (log_storage_warn, warn, common::log::log_targets::STORAGE),
    (log_storage_error, error, common::log::log_targets::STORAGE),
    // Application
    (log_app_trace, trace, common::log::log_targets::APPLICATION),
    (log_app_debug, debug, common::log::log_targets::APPLICATION),
    (log_app_info, info, common::log::log_targets::APPLICATION),
    (log_app_warn, warn, common::log::log_targets::APPLICATION),
    (log_app_error, error, common::log::log_targets::APPLICATION),
    // Authentication
    (
        log_auth_trace,
        trace,
        common::log::log_targets::AUTHENTICATION
    ),
    (
        log_auth_debug,
        debug,
        common::log::log_targets::AUTHENTICATION
    ),
    (
        log_auth_info,
        info,
        common::log::log_targets::AUTHENTICATION
    ),
    (
        log_auth_warn,
        warn,
        common::log::log_targets::AUTHENTICATION
    ),
    (
        log_auth_error,
        error,
        common::log::log_targets::AUTHENTICATION
    ),
    // HTTP
    (log_http_trace, trace, common::log::log_targets::HTTP),
    (log_http_debug, debug, common::log::log_targets::HTTP),
    (log_http_info, info, common::log::log_targets::HTTP),
    (log_http_warn, warn, common::log::log_targets::HTTP),
    (log_http_error, error, common::log::log_targets::HTTP),
];
