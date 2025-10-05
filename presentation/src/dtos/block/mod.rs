use crate::dtos::time::DateTimeExtPresentation;
use crate::dtos::transaction::TransactionPresentationDto;
use chrono::{DateTime, Utc};
use domain::entities::block::Block;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "Block")]
pub(crate) struct BlockPresentationDto {
    hash: String,
    data: InnerBlockPresentationDto,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "BlockData")]
struct InnerBlockPresentationDto {
    height: u64,
    prev_block_hash: Option<String>,
    nonce: u64,
    difficulty_target: u128,
    transactions_merkle_root: String,
    transactions: Vec<TransactionPresentationDto>,
    timestamp: DateTime<Utc>,
}

impl From<Block> for BlockPresentationDto {
    fn from(block: Block) -> Self {
        Self {
            hash: block.get_hash().to_string(),
            data: InnerBlockPresentationDto {
                height: block.get_height().as_u64(),
                prev_block_hash: block.get_prev_block_hash().map(|hash| hash.to_string()),
                nonce: block.get_nonce().as_u64(),
                difficulty_target: block.get_difficulty_target().as_u128(),
                transactions_merkle_root: block.get_transactions_merkle_root().to_string(),
                transactions: block
                    .get_transactions()
                    .iter()
                    .cloned()
                    .map(|tx| tx.into())
                    .collect(),
                timestamp: block.get_timestamp().to_chrono(),
            },
        }
    }
}
