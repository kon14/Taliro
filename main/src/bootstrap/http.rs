use application::state::AppState;
use common::config::http::HttpConfig;
use common::error::AppError;
use domain::system::network::validator::NetworkEntityValidator;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use presentation::utils::BuildHttpServerResponse;
use std::sync::Arc;
use tokio::sync::broadcast;

pub(crate) async fn build_http_app_state(
    cfg: HttpConfig,
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    net_entity_validator: Arc<dyn NetworkEntityValidator>,
) -> Result<AppState, AppError> {
    // Authentication
    let master_key_authenticator = Arc::new(
        infrastructure::auth::master_key::DefaultMasterKeyAuthenticator::new(cfg.master_key_secret),
    );

    // App State
    let app_state = AppState::new(
        cmd_tx,
        cmd_tx_res_factory,
        master_key_authenticator,
        net_entity_validator,
    );
    Ok(app_state)
}

pub(crate) async fn build_http_server(
    cfg: HttpConfig,
    app_state: AppState,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<BuildHttpServerResponse, AppError> {
    presentation::utils::build_http_server(cfg, app_state, shutdown_rx).await
}
