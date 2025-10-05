use crate::types::outbox::OutboxEntry;
use common::error::AppError;
use common::tx::ctx::AtomicTransactionContext;
use std::fmt::Debug;

#[cfg_attr(test, mockall::automock)]
pub trait OutboxRepository: Send + Sync + Debug {
    fn insert_entry<'a>(
        &self,
        tx_ctx: Option<&'a dyn AtomicTransactionContext>,
        entry: OutboxEntry,
    ) -> Result<(), AppError>;

    fn get_unprocessed_entries(&self) -> Result<Vec<OutboxEntry>, AppError>;

    fn mark_entry_as_processed(&self, entry: &OutboxEntry) -> Result<(), AppError>;
}
