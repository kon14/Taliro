use application::outbox::OutboxRelay;
use common::log_app_info;
use std::time::Duration;
use tokio::time::sleep;

pub(crate) fn spawn_outbox_relay_worker_task(
    outbox_relay: OutboxRelay,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    let poll_interval = Duration::from_secs(1); // TODO: make configurable

    tokio::spawn(async move {
        loop {
            // Handle shutdown signal
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            // Perform outbox processing
            outbox_relay.handle_unprocessed().await;

            tokio::select! {
                // Exit fast on shutdown signal
                _ = shutdown_rx.recv() => break,
                _ = sleep(poll_interval) => {},
            }
        }

        log_app_info!("Outbox relay worker task exiting...");
    });
}
