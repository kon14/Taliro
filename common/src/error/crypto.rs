use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptographicError {
    #[error("Hash length mismatch: expected {expected}, got {actual}")]
    HashLengthMismatch { expected: usize, actual: usize },

    #[error("Hash conversion failed: {reason}")]
    HashConversionFailed { reason: String },

    #[error("Encoding failed: {reason}")]
    EncodingFailed { reason: String },

    #[error("Decoding failed: {reason}")]
    DecodingFailed { reason: String },

    #[error("Signature verification failed")]
    SignatureVerificationFailed,
}
