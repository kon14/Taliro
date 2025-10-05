#[cfg(test)]
mod tests;

use crate::entities::transaction::{TransactionOutPoint, Utxo};
use crate::repos::utxo::UtxoRepository;
use common::error::AppError;
use std::fmt::Debug;
use std::sync::Arc;

pub trait UtxoSetReader: Send + Sync + Debug {
    fn get_utxo(&self, outpoint: &TransactionOutPoint) -> Result<Option<Utxo>, AppError>;

    fn get_multiple_utxos_by_outpoints(
        &self,
        outpoints: &Vec<TransactionOutPoint>,
    ) -> Result<Vec<Utxo>, AppError>;

    fn get_multiple_utxos(&self) -> Result<Vec<Utxo>, AppError>;

    fn get_utxo_count(&self) -> usize;
}

#[derive(Debug)]
pub(crate) struct UtxoReaderService {
    utxo_repo: Arc<dyn UtxoRepository>,
}

impl UtxoReaderService {
    pub fn new(utxo_repo: Arc<dyn UtxoRepository>) -> Self {
        Self { utxo_repo }
    }
}

impl UtxoSetReader for UtxoReaderService {
    fn get_utxo(&self, outpoint: &TransactionOutPoint) -> Result<Option<Utxo>, AppError> {
        let output = self.utxo_repo.get_output(None, outpoint)?;
        let utxo = output.map(|tx_out| Utxo::new(outpoint.clone(), tx_out));
        Ok(utxo)
    }

    fn get_multiple_utxos_by_outpoints(
        &self,
        outpoints: &Vec<TransactionOutPoint>,
    ) -> Result<Vec<Utxo>, AppError> {
        let utxos = outpoints
            .iter()
            .map(|outpoint| (outpoint, self.utxo_repo.get_output(None, outpoint)))
            .filter_map(|(outpoint, res_output)| match res_output {
                Ok(Some(output)) => Some(Ok(Utxo::new(outpoint.clone(), output))),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<Utxo>, AppError>>()?;

        Ok(utxos)
    }

    fn get_multiple_utxos(&self) -> Result<Vec<Utxo>, AppError> {
        self.utxo_repo.get_multiple_utxos()
    }

    fn get_utxo_count(&self) -> usize {
        self.utxo_repo.get_utxo_count()
    }
}
