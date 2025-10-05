use crate::ext::{AppErrorExtInfrastructure, TransactionContextExtInfrastructure};
use crate::storage::SledStorage;
use crate::tx::SledUnitOfWork;
use crate::tx::trees::{SledTxBlockchainAppendBlockTrees, SledTxTrees};
use common::error::AppError;
use common::tx::UnitOfWork;
use common::tx::ctx::AtomicTransactionContext;
use domain::encode::{TryDecode, TryEncode};
use domain::entities::block::{Block, BlockHeight};
use domain::repos::blockchain::BlockchainRepository;
use domain::types::hash::Hash;
use sled::Tree;
use std::fmt::{Debug, Formatter};
use std::ops::RangeInclusive;
use std::sync::Arc;

pub struct SledBlockchainRepository {
    blocks_tree: Tree,
    heights_tree: Tree,
    meta_tree: Tree,
    outbox_unprocessed_tree: Tree,
}

impl Debug for SledBlockchainRepository {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SledBlockchainRepository")
            .field("blocks_tree", &SledStorage::BLOCKCHAIN_BLOCKS_TREE)
            .field("heights_tree", &SledStorage::BLOCKCHAIN_HEIGHTS_TREE)
            .field("meta_tree", &SledStorage::BLOCKCHAIN_META_TREE)
            .finish()
    }
}

impl SledBlockchainRepository {
    pub fn open(
        blocks_tree: Tree,
        heights_tree: Tree,
        meta_tree: Tree,
        outbox_unprocessed_tree: Tree,
    ) -> Result<Self, AppError> {
        let repo = Self {
            blocks_tree,
            heights_tree,
            meta_tree,
            outbox_unprocessed_tree,
        };
        Ok(repo)
    }
}

impl BlockchainRepository for SledBlockchainRepository {
    fn get_blockchain_append_block_unit_of_work(&self) -> Arc<dyn UnitOfWork> {
        let trees = SledTxBlockchainAppendBlockTrees {
            blocks_tree: self.blocks_tree.clone(),
            heights_tree: self.heights_tree.clone(),
            outbox_unprocessed_tree: self.outbox_unprocessed_tree.clone(),
        };
        let trees = SledTxTrees::BlockchainAppendBlock(trees);
        Arc::new(SledUnitOfWork::new(trees))
    }

    fn insert_block(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        block: &Block,
    ) -> Result<(), AppError> {
        let data = block.try_encode()?;
        if let Some(tx_ctx) = tx_ctx {
            let blocks_tree = tx_ctx.get_blocks_tree()?;
            blocks_tree
                .insert(block.get_hash().as_ref(), data)
                .to_app_error()?;
        } else {
            self.blocks_tree
                .insert(block.get_hash().as_ref(), data)
                .to_app_error()?;
        }
        Ok(())
    }

    fn get_block(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        hash: &Hash,
    ) -> Result<Option<Block>, AppError> {
        let block = if let Some(tx_ctx) = tx_ctx {
            let blocks_tree = tx_ctx.get_blocks_tree()?;
            blocks_tree.get(hash.as_ref()).to_app_error()?
        } else {
            self.blocks_tree.get(hash.as_ref()).to_app_error()?
        };
        if let Some(bytes) = block {
            let block = Block::try_decode(&bytes)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    fn get_multiple_blocks(&self, hashes: Vec<Hash>) -> Result<Vec<Block>, AppError> {
        let blocks = hashes
            .into_iter()
            .map(|hash| self.get_block(None, &hash))
            .filter_map(|res| match res {
                Ok(Some(block)) => Some(Ok(block)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<Block>, AppError>>()?;
        Ok(blocks)
    }

    fn get_block_hash_by_height(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        height: &BlockHeight,
    ) -> Result<Option<Hash>, AppError> {
        let height_key = height.to_be_bytes();
        let height = if let Some(tx_ctx) = tx_ctx {
            let heights_tree = tx_ctx.get_heights_tree()?;
            heights_tree.get(height_key).to_app_error()?
        } else {
            self.heights_tree.get(height_key).to_app_error()?
        };
        if let Some(bytes) = height {
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(&bytes);
            Ok(Some(Hash::new(hash_bytes)))
        } else {
            Ok(None)
        }
    }

    fn get_block_hashes_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> Result<Vec<Hash>, AppError> {
        let start = height_range.start().to_be_bytes();
        let end = height_range.end().to_be_bytes();

        let hashes = self
            .heights_tree
            .range(start..=end)
            .map(|res| {
                let (_, value) =
                    res.map_err(|err| AppError::internal(format!("Storage error: {}", err)))?;
                if value.len() != 32 {
                    return Err(AppError::internal("Invalid block hash!"));
                }
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(&value);
                Ok(Hash::new(hash_bytes))
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        Ok(hashes)
    }

    fn get_height(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        hash: &Hash,
    ) -> Result<Option<BlockHeight>, AppError> {
        // Optimization: Maintain a secondary block_hash -> height index.
        match self.get_block(tx_ctx, hash)? {
            Some(block) => Ok(Some(block.get_height())),
            None => Ok(None),
        }
    }

    fn insert_height(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        height: BlockHeight,
        block_hash: &Hash,
    ) -> Result<(), AppError> {
        let height_key = height.to_be_bytes();
        if let Some(tx_ctx) = tx_ctx {
            let heights_tree = tx_ctx.get_heights_tree()?;
            heights_tree
                .insert(&height_key, block_hash.as_ref())
                .to_app_error()?;
        } else {
            self.heights_tree
                .insert(&height_key, block_hash.as_ref())
                .to_app_error()?;
        };
        Ok(())
    }

    fn get_tip(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
    ) -> Result<Option<Hash>, AppError> {
        let tip = if let Some(tx_ctx) = tx_ctx {
            let meta_tree = tx_ctx.get_meta_tree()?;
            meta_tree
                .get(SledStorage::BLOCKCHAIN_META_TREE_TIP_KEY)
                .to_app_error()?
        } else {
            self.meta_tree
                .get(SledStorage::BLOCKCHAIN_META_TREE_TIP_KEY)
                .to_app_error()?
        };
        if let Some(bytes) = tip {
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(&bytes);
            Ok(Some(Hash::new(hash_bytes)))
        } else {
            Ok(None)
        }
    }

    fn set_tip(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        hash: &Hash,
    ) -> Result<(), AppError> {
        if let Some(tx_ctx) = tx_ctx {
            let meta_tree = tx_ctx.get_meta_tree()?;
            meta_tree
                .insert(SledStorage::BLOCKCHAIN_META_TREE_TIP_KEY, hash.as_ref())
                .to_app_error()?;
        } else {
            self.meta_tree
                .insert(SledStorage::BLOCKCHAIN_META_TREE_TIP_KEY, hash.as_ref())
                .to_app_error()?;
        };
        Ok(())
    }
}
