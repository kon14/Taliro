use thiserror::Error;

// TODO: use tx_id in Display

#[derive(Error, Debug)]
pub enum TransactionValidationError {
    #[error("Input UTXO not found: {outpoint}")]
    InputUtxoNotFound { tx_id: String, outpoint: String },

    #[error("Double spending detected for outpoint: {outpoint}")]
    DoubleSpending { tx_id: String, outpoint: String },

    #[error("Transaction outputs ({outputs}) exceed inputs ({inputs})")]
    OutputsExceedInputs {
        tx_id: String,
        inputs: u128,
        outputs: u128,
    },

    #[error("Invalid signature for input {index}")]
    InvalidSignature { tx_id: String, index: usize },

    #[error("Zero or negative amount in output {index}")]
    InvalidOutputAmount { tx_id: String, index: usize },

    #[error("Transaction must have at least one input")]
    EmptyInputs { tx_id: String },

    #[error("Transaction must have at least one output")]
    EmptyOutputs { tx_id: String },
}
