use application::state::AppState;
use common::config::http::HttpConfig;
use common::error::AppError;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use presentation::utils::BuildHttpServerResponse;
use std::sync::Arc;
use tokio::sync::broadcast;

pub(crate) async fn build_http_app_state(
    cfg: HttpConfig,
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
) -> Result<AppState, AppError> {
    // Authentication
    let master_key_authenticator = Arc::new(
        infrastructure::auth::master_key::DefaultMasterKeyAuthenticator::new(cfg.master_key_secret),
    );

    // App State
    let app_state = AppState::new(bus_tx, bus_tx_res_factory, master_key_authenticator);
    Ok(app_state)
}

pub(crate) async fn build_http_server(
    cfg: HttpConfig,
    app_state: AppState,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<BuildHttpServerResponse, AppError> {
    presentation::utils::build_http_server(cfg, app_state, shutdown_rx).await
}
