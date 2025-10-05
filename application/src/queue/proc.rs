use async_trait::async_trait;
use common::log_app_debug;
use domain::entities::block::{BlockHeight, NonValidatedBlock};
use domain::system::queue::BlockProcessingQueue;
use std::collections::{BTreeMap, HashSet};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct DefaultBlockProcessingQueue {
    next_expected: Mutex<BlockHeight>,
    buffered_blocks: Mutex<BTreeMap<BlockHeight, NonValidatedBlock>>,
    in_flight: Mutex<HashSet<BlockHeight>>,
}

impl DefaultBlockProcessingQueue {
    pub fn new(next_expected_height: BlockHeight) -> Self {
        Self {
            next_expected: Mutex::new(next_expected_height),
            buffered_blocks: Mutex::new(BTreeMap::new()),
            in_flight: Mutex::new(HashSet::new()),
        }
    }
}

#[async_trait]
impl BlockProcessingQueue for DefaultBlockProcessingQueue {
    async fn push_block(&self, block: NonValidatedBlock) {
        let next_expected = self.next_expected.lock().await;
        let mut buffered = self.buffered_blocks.lock().await;
        let height = block.get_height();

        if height == *next_expected {
            // We could process this one immediately, but that would introduce coupling.
            // Let's buffer and rely on polling instead...
            buffered.insert(height, block);
        } else if height > *next_expected {
            buffered.insert(height, block);
        } else {
            log_app_debug!(
                "BlockProcessingQueue.push_block() | Ignoring old or duplicate block {:?}",
                height
            );
        }
    }

    async fn next_ready_block(&self) -> Option<NonValidatedBlock> {
        let next_expected = self.next_expected.lock().await;
        let buffered = self.buffered_blocks.lock().await;
        let mut in_flight = self.in_flight.lock().await;

        let next_block = buffered.get(&next_expected);
        if let Some(block) = next_block {
            let height = block.get_height();

            if in_flight.contains(&height) {
                return None;
            }

            in_flight.insert(height.clone());
            return Some(block.clone());
        }
        None
    }

    async fn mark_block_processed(&self, height: &BlockHeight) {
        let mut next_expected = self.next_expected.lock().await;
        let mut buffered = self.buffered_blocks.lock().await;
        let mut in_flight = self.in_flight.lock().await;

        buffered.remove(height);
        in_flight.remove(height);

        if *next_expected == *height {
            *next_expected = next_expected.next();
        }
    }

    async fn mark_block_failed(&self, height: &BlockHeight) {
        let mut in_flight = self.in_flight.lock().await;
        in_flight.remove(height);
    }
}
