#[cfg(test)]
mod tests;

use crate::entities::transaction::{NonValidatedTransaction, Transaction, TransactionAmount, Utxo};
use crate::system::utxo::UtxoSetReader;
use async_trait::async_trait;
use common::error::{AppError, TransactionValidationError};
use std::sync::Arc;

// Note: This could technically be split into offline/online validators.

/// Ensures <strong>individual</strong> transactions are valid according to consensus rules.<br />
/// Does <strong>NOT</strong> handle block-level transaction validations!
#[async_trait]
pub(crate) trait TransactionValidator: Send + Sync + std::fmt::Debug {
    async fn validate_transaction(
        &self,
        tx: NonValidatedTransaction,
    ) -> Result<Transaction, AppError>;
}

#[derive(Debug)]
pub(crate) struct DefaultTransactionValidator {
    utxo_set_r: Arc<dyn UtxoSetReader>,
}

#[async_trait]
impl TransactionValidator for DefaultTransactionValidator {
    async fn validate_transaction(
        &self,
        tx: NonValidatedTransaction,
    ) -> Result<Transaction, AppError> {
        self.validate_structure(&tx)?;
        self.validate_signatures(&tx)?;

        let input_utxos = self.validate_inputs_unspent(&tx).await?;
        // self.validate_input_values(&tx, &input_utxos)?;
        self.validate_output_values(&tx)?;
        self.validate_balance(&tx, &input_utxos)?;

        Ok(Transaction::_new_validated(tx))
    }
}

impl DefaultTransactionValidator {
    pub fn new(utxo_set_r: Arc<dyn UtxoSetReader>) -> Self {
        Self { utxo_set_r }
    }

    fn validate_structure(&self, tx: &NonValidatedTransaction) -> Result<(), AppError> {
        if tx.get_inputs().is_empty() {
            return Err(AppError::TransactionValidation(
                TransactionValidationError::EmptyInputs {
                    tx_id: tx.get_hash().to_string(),
                },
            ));
        }
        if tx.get_outputs().is_empty() {
            return Err(AppError::TransactionValidation(
                TransactionValidationError::EmptyOutputs {
                    tx_id: tx.get_hash().to_string(),
                },
            ));
        }
        Ok(())
    }

    #[allow(unused)]
    fn validate_signatures(&self, tx: &NonValidatedTransaction) -> Result<(), AppError> {
        // Verify all input signatures using public keys and prevout scripts
        // TODO
        Ok(())
    }

    async fn validate_inputs_unspent(
        &self,
        tx: &NonValidatedTransaction,
    ) -> Result<Vec<Utxo>, AppError> {
        let mut utxos = Vec::new();
        for txin in tx.get_inputs() {
            let outpoint = txin.get_previous_output();
            let Some(utxo) = self.utxo_set_r.get_utxo(outpoint)? else {
                return Err(AppError::TransactionValidation(
                    TransactionValidationError::InputUtxoNotFound {
                        tx_id: tx.get_hash().to_string(),
                        outpoint: outpoint.to_string(),
                    },
                ));
            };
            utxos.push(utxo);
        }
        Ok(utxos)
    }

    fn validate_output_values(&self, tx: &NonValidatedTransaction) -> Result<(), AppError> {
        for (index, txout) in tx.get_outputs().iter().enumerate() {
            if txout.get_amount() <= TransactionAmount::new(0) {
                return Err(AppError::TransactionValidation(
                    TransactionValidationError::InvalidOutputAmount {
                        tx_id: tx.get_hash().to_string(),
                        index,
                    },
                ));
            }
        }
        Ok(())
    }

    fn validate_balance(
        &self,
        tx: &NonValidatedTransaction,
        fetched_input_utxos: &[Utxo],
    ) -> Result<(), AppError> {
        let mut input_sum = TransactionAmount::new(0);
        let mut output_sum = TransactionAmount::new(0);

        for utxo in fetched_input_utxos {
            input_sum.checked_add_assign(utxo.get_output().get_amount())?;
        }

        for txout in tx.get_outputs() {
            output_sum.checked_add_assign(txout.get_amount())?;
        }

        if input_sum < output_sum {
            return Err(AppError::TransactionValidation(
                TransactionValidationError::OutputsExceedInputs {
                    tx_id: tx.get_hash().to_string(),
                    inputs: input_sum.as_u128(),
                    outputs: output_sum.as_u128(),
                },
            ));
        }

        Ok(())
    }
}

/// Expose internal methods for unit testing.
#[cfg(test)]
impl DefaultTransactionValidator {
    pub(super) fn pub_validate_structure(
        &self,
        tx: &NonValidatedTransaction,
    ) -> Result<(), AppError> {
        self.validate_structure(tx)
    }

    #[allow(unused)]
    pub(super) fn pub_validate_signatures(
        &self,
        tx: &NonValidatedTransaction,
    ) -> Result<(), AppError> {
        self.validate_signatures(tx)
    }

    pub(super) async fn pub_validate_inputs_unspent(
        &self,
        tx: &NonValidatedTransaction,
    ) -> Result<Vec<Utxo>, AppError> {
        self.validate_inputs_unspent(tx).await
    }

    pub(super) fn pub_validate_output_values(
        &self,
        tx: &NonValidatedTransaction,
    ) -> Result<(), AppError> {
        self.validate_output_values(tx)
    }

    pub(super) fn pub_validate_balance(
        &self,
        tx: &NonValidatedTransaction,
        fetched_input_utxos: &[Utxo],
    ) -> Result<(), AppError> {
        self.validate_balance(tx, fetched_input_utxos)
    }
}
