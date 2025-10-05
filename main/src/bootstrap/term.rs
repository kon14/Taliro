use common::log_app_info;
use domain::system::node::bus::{CommandSender, NodeCommandRequest};
use std::sync::Arc;
use tokio::sync::broadcast;

#[cfg(unix)]
use tokio::signal::unix::{signal as unix_signal, SignalKind};

pub(crate) fn handle_termination_signals(
    bus_tx: Arc<dyn CommandSender>,
) -> (broadcast::Sender<()>, broadcast::Receiver<()>) {
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(16);

    tokio::spawn(async move {
        #[cfg(unix)]
        let mut sigterm_stream =
            unix_signal(SignalKind::terminate()).expect("Failed to bind to SIGTERM");

        #[cfg(unix)]
        let sigterm_fut = sigterm_stream.recv();

        #[cfg(not(unix))]
        let sigterm_fut = std::future::pending::<Option<()>>();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log_app_info!("Received SIGINT signal!");
            },
            _ = sigterm_fut => {
                log_app_info!("Received SIGTERM signal!");
            }
        }

        let _ = bus_tx.send(NodeCommandRequest::RequestNodeShutdown).await;
    });

    (shutdown_tx, shutdown_rx)
}
