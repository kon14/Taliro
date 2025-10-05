use crate::entities::block::{Block, BlockHeight};
use crate::types::hash::Hash;
use common::error::AppError;
use common::tx::UnitOfWork;
use common::tx::ctx::AtomicTransactionContext;
use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::sync::Arc;

pub trait BlockchainRepository: Send + Sync + Debug {
    fn get_blockchain_append_block_unit_of_work(&self) -> Arc<dyn UnitOfWork>;

    fn insert_block(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        block: &Block,
    ) -> Result<(), AppError>;

    fn get_block(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        hash: &Hash,
    ) -> Result<Option<Block>, AppError>;

    /// Retrieves multiple blocks by their hashes.
    /// This will raise an error if any hash doesn't have a corresponding block.
    fn get_multiple_blocks(&self, hashes: Vec<Hash>) -> Result<Vec<Block>, AppError>;

    fn get_block_hash_by_height(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        height: &BlockHeight,
    ) -> Result<Option<Hash>, AppError>;

    /// Retrieves block hashes within a range of heights (inclusive start, inclusive end).<br />
    /// This will raise an error if any height in the range doesn't have a corresponding block hash.
    fn get_block_hashes_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> Result<Vec<Hash>, AppError>;

    fn get_height(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        hash: &Hash,
    ) -> Result<Option<BlockHeight>, AppError>;

    fn insert_height(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        height: BlockHeight,
        block_hash: &Hash,
    ) -> Result<(), AppError>;

    fn get_tip(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
    ) -> Result<Option<Hash>, AppError>;

    fn set_tip(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        hash: &Hash,
    ) -> Result<(), AppError>;
}
