use axum::{extract::Request, ServiceExt};
use std::future::Future;
use std::pin::Pin;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::{http::build_router, types::http::ServerAddress};
use application::state::AppState;
use common::config::http::HttpConfig;
use common::error::AppError;
use common::log_http_info;

// TODO: bind to multiple interfaces (IPv4 + IPv6)
fn get_server_address(api_port: u16) -> String {
    format!("0.0.0.0:{}", api_port)
}

pub async fn build_http_server(
    cfg: HttpConfig,
    app_state: AppState,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<BuildHttpServerResponse, AppError> {
    let router = build_router(app_state.clone(), &cfg.api_base_url);

    let server_addr = get_server_address(cfg.api_port);
    let listener = TcpListener::bind(&server_addr).await.map_err(|err| {
        AppError::internal_with_private(
            format!("Failed to bind TCP listener @ {server_addr}"),
            err.to_string(),
        )
    })?;

    let serve_future = axum::serve(listener, ServiceExt::<Request>::into_make_service(router))
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
            log_http_info!("HTTP server worker task exiting...");
        });

    let address = server_addr.clone();
    let server = Box::pin(async move {
        log_http_info!("Server listening on: http://{address}");
        serve_future
            .await
            .map_err(|err| AppError::internal(err.to_string()))
    });

    Ok(BuildHttpServerResponse {
        server,
        server_addr: ServerAddress(server_addr),
    })
}

pub struct BuildHttpServerResponse {
    pub server: Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    pub server_addr: ServerAddress,
}
