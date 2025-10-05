use crate::entities::block::{BlockHeight, NonValidatedBlock};
use crate::types::network::NetworkPeerId;
use async_trait::async_trait;
use common::error::AppError;

/// A queue responsible for managing block-related network fetch events.<br />
/// Prevents redundant requests and coordinates sync operations.
#[async_trait]
pub trait BlockSyncQueue: Send + Sync + std::fmt::Debug {
    async fn request_block(
        &self,
        height: BlockHeight,
        from_peer: NetworkPeerId,
    ) -> Result<(), AppError>;

    async fn on_block_received(&self, block: NonValidatedBlock, from_peer: NetworkPeerId);

    async fn is_in_progress(&self, height: &BlockHeight) -> bool;
}
