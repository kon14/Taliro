use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Generic storage error: {reason}")]
    GenericError { reason: String },

    #[error("Database transaction failed: {reason}")]
    TransactionFailed { reason: String },

    #[error("Read operation failed: {reason}")]
    ReadFailed { reason: String },

    #[error("Write operation failed: {reason}")]
    WriteFailed { reason: String },

    #[error("Invalid transaction context")]
    InvalidTransactionContext,
}
