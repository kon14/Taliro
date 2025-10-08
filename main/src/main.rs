mod bootstrap;
mod ext;

use crate::ext::config::{PartialAppConfigFromEnvExtMain, PartialAppConfigFromFileExtMain};
use application::storage::Storage;
use common::config;
use common::config::AppConfig;
use common::error::AppError;
use infrastructure::cmd::NodeCommandResponderFactory;
use infrastructure::network::validator::Libp2pNetworkEntityValidator;
use infrastructure::storage::SledStorage;
use presentation::utils::BuildHttpServerResponse;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Setup Environment
    bootstrap::env::setup_env();

    // Load Application Configuration
    let file_cfg = config::PartialAppConfig::load_from_file()?;
    let env_cfg = config::PartialAppConfig::load_from_env()?;
    let AppConfig {
        http: http_config,
        network: network_config,
        node: node_config,
        storage: storage_config,
    } = AppConfig::from_parts(file_cfg, env_cfg)?;

    // Initialize Storage
    let storage = Box::new(SledStorage::open(storage_config)?);
    let network_repo = storage.get_network_repo();

    // Bring up Event Channel
    let (cmd_tx, cmd_rx) = bootstrap::cmd::build_cmd_channel();
    let cmd_tx_res_factory = Arc::new(NodeCommandResponderFactory);

    // Handle Termination Signals
    let (shutdown_tx, shutdown_rx) = bootstrap::term::handle_termination_signals(cmd_tx.clone());

    // Prepare the P2P Network
    let net_entity_validator = Arc::new(Libp2pNetworkEntityValidator);
    let network = bootstrap::network::build_p2p_network(
        network_config,
        network_repo,
        net_entity_validator.clone(),
    )?;

    // Prepare the Blockchain Node
    let (node, outbox_relay, block_sync_queue, block_proc_queue) = bootstrap::node::build_node(
        node_config,
        storage,
        network,
        cmd_tx.clone(),
        cmd_tx_res_factory.clone(),
    )
    .await?;
    let node = node
        .bootstrap(
            cmd_tx.clone(),
            cmd_tx_res_factory.clone(),
            block_sync_queue,
            block_proc_queue,
            shutdown_rx.resubscribe(),
        )
        .await?
        .start()?;

    // Start Outbox Relay
    bootstrap::outbox::spawn_outbox_relay_worker_task(outbox_relay, shutdown_rx.resubscribe());

    let node_run_fut = node.run(cmd_rx, shutdown_tx, shutdown_rx.resubscribe());

    // Prepare the HTTP Server
    let app_state = bootstrap::http::build_http_app_state(
        http_config.clone(),
        cmd_tx,
        cmd_tx_res_factory,
        net_entity_validator,
    )
    .await?;
    let BuildHttpServerResponse {
        server: server_run_fut,
        ..
    } = bootstrap::http::build_http_server(http_config, app_state, shutdown_rx).await?;

    // Run the Blockchain Node and the HTTP Server
    let (server_res, node_res) = tokio::join!(server_run_fut, node_run_fut);
    server_res?;
    node_res?;
    Ok(())
}
