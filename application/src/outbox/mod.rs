use common::error::AppError;
use common::{log_app_error, log_app_trace};
use domain::repos::outbox::OutboxRepository;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::types::outbox::{OutboxEntry, OutboxEvent};
use std::sync::Arc;

#[derive(Debug)]
pub struct OutboxRelay {
    outbox_repo: Arc<dyn OutboxRepository>,
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl OutboxRelay {
    pub fn new(
        outbox_repo: Arc<dyn OutboxRepository>,
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            outbox_repo,
            cmd_tx,
            cmd_tx_res_factory,
        }
    }

    pub async fn handle_unprocessed(&self) {
        match self.outbox_repo.get_unprocessed_entries() {
            Ok(entries) => {
                for entry in entries {
                    log_app_trace!("Processing outbox entry: {:?}", entry);
                    if let Err(err) = self.handle_event(entry.clone()).await {
                        log_app_error!(
                            "Failed to handle outbox entry ({}): {:?}",
                            entry.get_id(),
                            err
                        );
                        continue;
                    }

                    if let Err(err) = self.outbox_repo.mark_entry_as_processed(&entry) {
                        log_app_error!("Failed to mark entry as processed: {:?}", err);
                    }
                }
            }
            Err(err) => {
                log_app_error!("Failed to get unprocessed entries: {:?}", err);
            }
        }
    }

    async fn handle_event(&self, entry: OutboxEntry) -> Result<(), AppError> {
        match entry.get_event().clone() {
            OutboxEvent::BlockchainAppendBlock(block) => {
                let (command, res_fut) = self
                    .cmd_tx_res_factory
                    .build_blk_cmd_handle_block_append(block);
                self.cmd_tx.send(command).await?;
                res_fut.await
            }
        }
    }
}
