use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusValidationError {
    #[error("Block hash does not meet difficulty target")]
    InsufficientProofOfWork,

    #[error("Invalid difficulty adjustment")]
    InvalidDifficultyAdjustment,

    #[error("Invalid nonce")]
    InvalidNonce,

    #[error("Mining target not met")]
    MiningTargetNotMet,
}
