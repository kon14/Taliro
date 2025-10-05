mod base;
mod block;
mod consensus;
mod crypto;
mod network;
mod storage;
mod transaction;

pub use crate::error::base::BaseError;
pub use crate::error::block::BlockValidationError;
pub use crate::error::consensus::ConsensusValidationError;
pub use crate::error::crypto::CryptographicError;
pub use crate::error::network::NetworkError;
pub use crate::error::storage::StorageError;
pub use crate::error::transaction::TransactionValidationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Block validation failed: {0}")]
    BlockValidation(BlockValidationError),

    #[error("Transaction validation failed: {0}")]
    TransactionValidation(TransactionValidationError),

    #[error("Consensus validation failed: {0}")]
    ConsensusValidation(ConsensusValidationError),

    #[error("Cryptographic error: {0}")]
    Cryptographic(CryptographicError),

    #[error("Storage error: {0}")]
    Storage(StorageError),

    #[error("Network error: {0}")]
    Network(NetworkError),

    #[error("Configuration error: {0}")]
    Configuration(BaseError),

    #[error("Internal error: {0}")]
    Internal(BaseError),

    #[error("Bad Request: {0}")]
    BadRequest(BaseError),

    #[error("Not Found: {0}")]
    NotFound(BaseError),

    #[error("Unauthorized: {0}")]
    Unauthorized(BaseError),

    #[error("Forbidden: {0}")]
    Forbidden(BaseError),

    #[error("Conflict: {0}")]
    Conflict(BaseError),

    #[error("Precondition Failed: {0}")]
    PreconditionFailed(BaseError),
}

impl AppError {
    pub fn get_error_type(&self) -> &'static str {
        match self {
            AppError::BlockValidation(_) => "BlockValidation",
            AppError::TransactionValidation(_) => "TransactionValidation",
            AppError::ConsensusValidation(_) => "ConsensusValidation",
            AppError::Cryptographic(_) => "Cryptographic",
            AppError::Storage(_) => "Storage",
            AppError::Network(_) => "Network",
            AppError::Configuration(_) => "Configuration",
            AppError::Internal(_) => "Internal",
            AppError::BadRequest(_) => "BadRequest",
            AppError::NotFound(_) => "NotFound",
            AppError::Unauthorized(_) => "Unauthorized",
            AppError::Conflict(_) => "Conflict",
            AppError::Forbidden(_) => "Forbidden",
            AppError::PreconditionFailed(_) => "PreconditionFailed",
        }
    }

    pub fn get_public_info(&self) -> String {
        match self {
            AppError::BlockValidation(block_err) => block_err.to_string(),
            AppError::TransactionValidation(tx_err) => tx_err.to_string(),
            AppError::ConsensusValidation(consensus_err) => consensus_err.to_string(),
            AppError::Cryptographic(crypto_err) => crypto_err.to_string(),
            AppError::Storage(storage_err) => storage_err.to_string(),
            AppError::Network(net_err) => net_err.to_string(),
            AppError::Configuration(base_err) => base_err.to_string(),
            AppError::Internal(base_err) => base_err.to_string(),
            AppError::BadRequest(base_err) => base_err.to_string(),
            AppError::NotFound(base_err) => base_err.to_string(),
            AppError::Unauthorized(base_err) => base_err.to_string(),
            AppError::Conflict(base_err) => base_err.to_string(),
            AppError::Forbidden(base_err) => base_err.to_string(),
            AppError::PreconditionFailed(base_err) => base_err.to_string(),
        }
    }
}

impl AppError {
    pub fn internal<P>(public_info: P) -> Self
    where
        P: Into<String>,
    {
        Self::Internal(BaseError::new(public_info.into(), None))
    }

    pub fn internal_with_private<P, R>(public_info: P, private_info: R) -> Self
    where
        P: Into<String>,
        R: Into<String>,
    {
        Self::Internal(BaseError::new(
            public_info.into(),
            Some(private_info.into()),
        ))
    }

    pub fn bad_request<P>(public_info: P) -> Self
    where
        P: Into<String>,
    {
        Self::BadRequest(BaseError::new(public_info.into(), None))
    }

    pub fn bad_request_with_private<P, R>(public_info: P, private_info: R) -> Self
    where
        P: Into<String>,
        R: Into<String>,
    {
        Self::BadRequest(BaseError::new(
            public_info.into(),
            Some(private_info.into()),
        ))
    }

    pub fn not_found<P>(public_info: P) -> Self
    where
        P: Into<String>,
    {
        Self::NotFound(BaseError::new(public_info.into(), None))
    }

    pub fn not_found_with_private<P, R>(public_info: P, private_info: R) -> Self
    where
        P: Into<String>,
        R: Into<String>,
    {
        Self::NotFound(BaseError::new(
            public_info.into(),
            Some(private_info.into()),
        ))
    }

    pub fn unauthorized<P>(public_info: P) -> Self
    where
        P: Into<String>,
    {
        Self::Unauthorized(BaseError::new(public_info.into(), None))
    }

    pub fn unauthorized_with_private<P, R>(public_info: P, private_info: R) -> Self
    where
        P: Into<String>,
        R: Into<String>,
    {
        Self::Unauthorized(BaseError::new(
            public_info.into(),
            Some(private_info.into()),
        ))
    }

    pub fn conflict<P>(public_info: P) -> Self
    where
        P: Into<String>,
    {
        Self::Conflict(BaseError::new(public_info.into(), None))
    }

    pub fn conflict_with_private<P, R>(public_info: P, private_info: R) -> Self
    where
        P: Into<String>,
        R: Into<String>,
    {
        Self::Conflict(BaseError::new(
            public_info.into(),
            Some(private_info.into()),
        ))
    }

    pub fn forbidden<P>(public_info: P) -> Self
    where
        P: Into<String>,
    {
        Self::Forbidden(BaseError::new(public_info.into(), None))
    }

    pub fn forbidden_with_private<P, R>(public_info: P, private_info: R) -> Self
    where
        P: Into<String>,
        R: Into<String>,
    {
        Self::Forbidden(BaseError::new(
            public_info.into(),
            Some(private_info.into()),
        ))
    }

    pub fn precondition_failed<P>(public_info: P) -> Self
    where
        P: Into<String>,
    {
        Self::PreconditionFailed(BaseError::new(public_info.into(), None))
    }

    pub fn precondition_failed_with_private<P, R>(public_info: P, private_info: R) -> Self
    where
        P: Into<String>,
        R: Into<String>,
    {
        Self::PreconditionFailed(BaseError::new(
            public_info.into(),
            Some(private_info.into()),
        ))
    }
}
