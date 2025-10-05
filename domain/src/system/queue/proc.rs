use crate::entities::block::{BlockHeight, NonValidatedBlock};
use async_trait::async_trait;

/// A queue responsible for in-order block processing, buffering out-of-order blocks and preventing concurrent handling of the same block.
#[async_trait]
pub trait BlockProcessingQueue: Send + Sync + std::fmt::Debug {
    /// Push a block into the processing queue.<br />
    /// Should be polled by [`next_ready_block()`].
    async fn push_block(&self, block: NonValidatedBlock);

    /// Poll for the next block ready for processing.<br />
    /// Subsequent polls will return `None` until the previously returned block is marked as processed or failed.
    async fn next_ready_block(&self) -> Option<NonValidatedBlock>;

    /// Notify the queue that a block was successfully processed.
    async fn mark_block_processed(&self, height: &BlockHeight);

    /// Notify the queue that a block processing failed.<br />
    /// This will allow re-processing of the block in future polls.
    async fn mark_block_failed(&self, height: &BlockHeight);
}
