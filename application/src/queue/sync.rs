use async_trait::async_trait;
use common::error::AppError;
use common::log_app_debug;
use domain::entities::block::{BlockHeight, NonValidatedBlock};
use domain::system::network::event::{NetworkEvent, TaliroNetworkData, TaliroNetworkEvent};
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::system::queue::{BlockProcessingQueue, BlockSyncQueue};
use domain::types::network::NetworkPeerId;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct DefaultBlockSyncQueue {
    in_progress: Mutex<HashSet<BlockHeight>>,
    completed: Mutex<HashSet<BlockHeight>>,
    block_proc_queue: Arc<dyn BlockProcessingQueue>,
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl DefaultBlockSyncQueue {
    pub fn new(
        block_proc_queue: Arc<dyn BlockProcessingQueue>,
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            in_progress: Mutex::new(HashSet::new()),
            completed: Mutex::new(HashSet::new()),
            block_proc_queue,
            bus_tx,
            bus_tx_res_factory,
        }
    }

    async fn request_block_from_network(
        &self,
        height: BlockHeight,
        from_peer: NetworkPeerId,
    ) -> Result<(), AppError> {
        let event_data = TaliroNetworkData::GetBlockByHeight(height.clone());
        let event = NetworkEvent::Taliro(TaliroNetworkEvent::new(from_peer.clone(), event_data));
        let (command, _) = self
            .bus_tx_res_factory
            .build_proxy_cmd_forward_network_event(event);
        self.bus_tx.send(command).await?;
        Ok(())
    }
}

#[async_trait]
impl BlockSyncQueue for DefaultBlockSyncQueue {
    async fn request_block(
        &self,
        height: BlockHeight,
        from_peer: NetworkPeerId,
    ) -> Result<(), AppError> {
        let mut in_progress = self.in_progress.lock().await;
        let completed = self.completed.lock().await;

        if completed.contains(&height) || in_progress.contains(&height) {
            return Ok(());
        }

        in_progress.insert(height.clone());
        self.request_block_from_network(height.clone(), from_peer.clone())
            .await?;
        log_app_debug!(
            "BlockSyncQueue.request_block() | Requesting block (height: {:?}) from peer {:?}",
            height,
            from_peer
        );
        Ok(())
    }

    async fn on_block_received(&self, block: NonValidatedBlock, _from_peer: NetworkPeerId) {
        let mut in_progress = self.in_progress.lock().await;
        let mut completed = self.completed.lock().await;
        let height = block.get_height();

        if completed.contains(&height) {
            return;
        }

        completed.insert(height.clone());
        in_progress.remove(&height);

        let block_hash = block.get_hash();
        let block_height = block.get_height();
        log_app_debug!(
            "BlockSyncQueue.on_block_received() | Pushing block (hash: {block_hash:?}, height: {block_height:?}) to BlockProcessingQueue."
        );
        self.block_proc_queue.push_block(block).await;
    }

    async fn is_in_progress(&self, height: &BlockHeight) -> bool {
        let in_progress = self.in_progress.lock().await;
        in_progress.contains(height)
    }
}
