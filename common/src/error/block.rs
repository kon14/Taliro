use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockValidationError {
    #[error("Invalid merkle root: expected {expected}, got {actual}")]
    InvalidMerkleRoot { expected: String, actual: String },

    #[error("Block contains no transactions")]
    NoTransactions,

    #[error("Block contains duplicate transactions")]
    DuplicateTransactions,

    #[error("Genesis block already exists")]
    GenesisAlreadyExists,

    #[error(
        "Block continuity mismatch: block references {block_prev_hash:?} but tip is {blockchain_tip_hash:?}"
    )]
    ContinuityMismatch {
        block_prev_hash: Option<String>,
        blockchain_tip_hash: Option<String>,
    },

    #[error("Block already known: {hash}")]
    BlockAlreadyKnown { hash: String },

    #[error("Multiple coinbase transactions found")]
    MultipleCoinbaseTransactions,

    #[error("Invalid timestamp: {reason}")]
    InvalidTimestamp { reason: String },

    #[error("Block size exceeds limit: {size} > {limit}")]
    BlockSizeExceeded { size: usize, limit: usize },
}
