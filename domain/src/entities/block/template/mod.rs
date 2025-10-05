use crate::entities::block::Block;
use crate::entities::block::difficulty::BlockDifficultyTarget;
use crate::entities::block::height::BlockHeight;
use crate::entities::block::nonce::BlockNonce;
use crate::entities::transaction::NonValidatedTransaction;
use crate::types::hash::Hash;
use crate::types::time::DateTime;

#[derive(Debug)]
pub struct BlockTemplate {
    pub(super) block_height: BlockHeight,
    pub(super) prev_block_hash: Option<Hash>,
    pub(super) nonce: BlockNonce,
    pub(super) difficulty_target: BlockDifficultyTarget,
    pub(super) transactions: Vec<NonValidatedTransaction>,
    #[allow(unused)]
    pub(super) timestamp: DateTime,
}

impl BlockTemplate {
    pub fn new(
        prev_block: &Block,
        transactions: Vec<NonValidatedTransaction>,
        difficulty_target: BlockDifficultyTarget,
    ) -> Self {
        Self {
            block_height: BlockHeight(prev_block.data.height.0 + 1),
            prev_block_hash: Some(prev_block.hash.clone()),
            nonce: BlockNonce::default(),
            difficulty_target,
            transactions,
            timestamp: DateTime::now(),
        }
    }

    #[allow(unused)]
    pub(crate) fn increment_nonce(&mut self) {
        self.nonce.0 += 1;
        self.update_timestamp();
    }

    fn update_timestamp(&mut self) {
        self.timestamp = DateTime::now();
    }
}
