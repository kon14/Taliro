use crate::entities::block::{Block, BlockHeight};
use crate::repos::blockchain::BlockchainRepository;
use crate::repos::outbox::OutboxRepository;
use crate::types::hash::Hash;
use crate::types::outbox::{OutboxEntry, OutboxEvent};
use async_trait::async_trait;
use common::error::{AppError, BlockValidationError};
use common::log_blk_info;
use common::tx::AtomicTransactionOutput;
use std::ops::RangeInclusive;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub(crate) trait Blockchain: Send + Sync + std::fmt::Debug {
    async fn add_block(&self, block: Block) -> Result<(), AppError>;
    async fn set_tip(&self, block: &Block) -> Result<(), AppError>;
    async fn get_tip_info(&self) -> Result<Option<(Hash, BlockHeight)>, AppError>;
    async fn get_unknown_block_heights(
        &self,
        remote_tip_info: (Hash, BlockHeight),
    ) -> Result<Option<RangeInclusive<BlockHeight>>, AppError>;
    async fn has_canon_block(&self, hash: &Hash) -> Result<bool, AppError>;
    async fn has_known_block(&self, hash: &Hash) -> Result<bool, AppError>;
    async fn get_canon_block(&self, hash: &Hash) -> Result<Option<Block>, AppError>;
    async fn get_known_block(&self, hash: &Hash) -> Result<Option<Block>, AppError>;
    async fn get_canon_block_by_height(
        &self,
        hash: &BlockHeight,
    ) -> Result<Option<Block>, AppError>;
    async fn get_known_block_by_height(
        &self,
        hash: &BlockHeight,
    ) -> Result<Option<Block>, AppError>;
    async fn get_canon_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> Result<Vec<Block>, AppError>;
    async fn get_known_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> Result<Vec<Block>, AppError>;
}

#[derive(Debug)]
pub(crate) struct DefaultBlockchain {
    blockchain_repo: Arc<dyn BlockchainRepository>,
    outbox_repo: Arc<dyn OutboxRepository>,
    tip_cache: Mutex<Option<(Hash, BlockHeight)>>,
}

#[async_trait]
impl Blockchain for DefaultBlockchain {
    async fn add_block(&self, block: Block) -> Result<(), AppError> {
        log_blk_info!("Blockchain.add_block() | block: {:?}", &block);
        let block_hash = block.get_hash();

        // Ensure previously validated blocks remain aligned with the latest blockchain state.
        let tip_info = self.get_tip_info().await?;
        let block_prev_hash = block.get_prev_block_hash();
        if block_prev_hash.as_ref() != tip_info.as_ref().map(|tip_info| &tip_info.0) {
            return Err(AppError::BlockValidation(
                BlockValidationError::ContinuityMismatch {
                    block_prev_hash: block_prev_hash.map(|h| h.to_string()),
                    blockchain_tip_hash: tip_info.as_ref().map(|tip_info| tip_info.0.to_string()),
                },
            ));
        }

        self.append_block(block).await?;

        log_blk_info!(
            "Blockchain.add_block(): Blockchain successfully appended block ({}) ",
            block_hash
        );
        Ok(())
    }

    async fn set_tip(&self, block: &Block) -> Result<(), AppError> {
        log_blk_info!("Blockchain.set_tip() | block: {:?}", &block);

        let block_hash = block.get_hash();
        let block_height = block.get_height();
        self.blockchain_repo.set_tip(None, &block_hash)?;
        *self.tip_cache.lock().await = Some((block_hash, block_height));

        log_blk_info!(
            "Blockchain.set_tip(): Blockchain tip successfully incremented to block ({}) ",
            block.get_hash()
        );
        Ok(())
    }

    async fn get_tip_info(&self) -> Result<Option<(Hash, BlockHeight)>, AppError> {
        if let Some(cached) = self.tip_cache.lock().await.as_ref() {
            return Ok(Some(cached.clone()));
        }

        let Some(tip_hash) = self.blockchain_repo.get_tip(None)? else {
            return Ok(None);
        };
        let Some(tip_height) = self.blockchain_repo.get_height(None, &tip_hash)? else {
            return Ok(None);
        };

        let tip = (tip_hash, tip_height);
        *self.tip_cache.lock().await = Some(tip.clone());
        Ok(Some(tip))
    }

    async fn get_unknown_block_heights(
        &self,
        remote_tip_info: (Hash, BlockHeight),
    ) -> Result<Option<RangeInclusive<BlockHeight>>, AppError> {
        let (_, remote_tip_height) = remote_tip_info;
        let local_tip_info = self.get_tip_info().await?;

        let Some((_, local_tip_height)) = local_tip_info else {
            return Ok(Some(BlockHeight::genesis()..=remote_tip_height));
        };

        // TODO: handle reorgs...

        if remote_tip_height <= local_tip_height {
            return Ok(None);
        }

        Ok(Some(local_tip_height.next()..=remote_tip_height))
    }

    async fn has_canon_block(&self, hash: &Hash) -> Result<bool, AppError> {
        self.get_canon_block(hash).await.map(|b| b.is_some())
    }

    async fn has_known_block(&self, hash: &Hash) -> Result<bool, AppError> {
        self.blockchain_repo
            .get_block(None, hash)
            .map(|b| b.is_some())
    }

    async fn get_canon_block(&self, hash: &Hash) -> Result<Option<Block>, AppError> {
        let Some(block) = self.get_known_block(hash).await? else {
            return Ok(None);
        };
        self.block_if_canon(block).await
    }

    async fn get_known_block(&self, hash: &Hash) -> Result<Option<Block>, AppError> {
        self.blockchain_repo.get_block(None, hash)
    }

    async fn get_canon_block_by_height(
        &self,
        height: &BlockHeight,
    ) -> Result<Option<Block>, AppError> {
        let Some(block) = self.get_known_block_by_height(height).await? else {
            return Ok(None);
        };
        self.block_if_canon(block).await
    }

    async fn get_known_block_by_height(
        &self,
        height: &BlockHeight,
    ) -> Result<Option<Block>, AppError> {
        let hash = self
            .blockchain_repo
            .get_block_hash_by_height(None, height)?;
        let block = match hash {
            Some(hash) => self.blockchain_repo.get_block(None, &hash)?,
            None => None,
        };
        Ok(block)
    }

    async fn get_canon_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> Result<Vec<Block>, AppError> {
        let expected_len =
            (height_range.end().as_u64() - height_range.start().as_u64() + 1) as usize;

        let blocks = async {
            let Some((_, tip_height)) = self.get_tip_info().await? else {
                return Ok(Vec::new());
            };
            let canon_range = if *height_range.end() > tip_height {
                height_range.start().clone()..=tip_height
            } else {
                height_range
            };
            self.get_known_blocks_by_height_range(canon_range).await
        }
        .await?;

        if blocks.len() != expected_len {
            return Err(AppError::bad_request(format!(
                "Block count mismatch! Expected {} block(s), but only found {}.",
                expected_len,
                blocks.len(),
            )));
        }
        Ok(blocks)
    }

    async fn get_known_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> Result<Vec<Block>, AppError> {
        let expected_len =
            (height_range.end().as_u64() - height_range.start().as_u64() + 1) as usize;

        let hashes = self
            .blockchain_repo
            .get_block_hashes_by_height_range(height_range)?;
        if hashes.len() != expected_len {
            return Err(AppError::bad_request(format!(
                "Block hash count mismatch! Expected {} block hash(es), but only found {}.",
                expected_len,
                hashes.len(),
            )));
        }

        let blocks = self.blockchain_repo.get_multiple_blocks(hashes)?;
        if blocks.len() != expected_len {
            return Err(AppError::bad_request(format!(
                "Block count mismatch! Expected {} block(s), but only found {}.",
                expected_len,
                blocks.len(),
            )));
        }
        Ok(blocks)
    }
}

impl DefaultBlockchain {
    pub fn new(
        blockchain_repo: Arc<dyn BlockchainRepository>,
        outbox_repo: Arc<dyn OutboxRepository>,
    ) -> Self {
        Self {
            blockchain_repo,
            outbox_repo,
            tip_cache: Mutex::new(None),
        }
    }

    async fn block_if_canon(&self, block: Block) -> Result<Option<Block>, AppError> {
        let Some((tip_hash, tip_height)) = self.get_tip_info().await? else {
            return Ok(None);
        };
        if block.get_height() > tip_height {
            return Ok(None);
        }
        if block.get_height() < tip_height {
            // Forks not currently supported. Any block below tip is canon
            return Ok(Some(block));
        }
        if block.get_hash() == tip_hash {
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    async fn append_block(&self, block: Block) -> Result<(), AppError> {
        let hash = block.get_hash();
        let height = block.get_height();

        let blockchain_repo = self.blockchain_repo.clone();
        let outbox_repo = self.outbox_repo.clone();
        let outbox_entry = OutboxEntry::new(OutboxEvent::BlockchainAppendBlock(block.clone()));

        let unit_of_work = self
            .blockchain_repo
            .get_blockchain_append_block_unit_of_work();
        unit_of_work.run_in_transaction(Box::new(move |ctx| {
            blockchain_repo.insert_block(Some(ctx), &block)?;
            blockchain_repo.insert_height(Some(ctx), height.clone(), &hash)?;
            outbox_repo.insert_entry(Some(ctx), outbox_entry.clone())?;
            Ok(AtomicTransactionOutput::new(()))
        }))?;
        Ok(())
    }
}
