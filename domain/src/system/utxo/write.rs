use crate::entities::block::Block;
use crate::entities::transaction::{TransactionOutPoint, Utxo};
use crate::repos::utxo::UtxoRepository;
use common::error::AppError;
use common::tx::AtomicTransactionOutput;
use common::{log_utxo_debug, log_utxo_info};
use std::fmt::Debug;
use std::sync::Arc;

pub trait UtxoSetWriter: Send + Sync + Debug {
    fn apply_block(&self, block: &Block) -> Result<(), AppError>;
}

#[derive(Debug)]
pub(crate) struct UtxoSetWriterService {
    utxo_repo: Arc<dyn UtxoRepository>,
}

impl UtxoSetWriterService {
    pub fn new(utxo_repo: Arc<dyn UtxoRepository>) -> Self {
        Self { utxo_repo }
    }
}

impl UtxoSetWriter for UtxoSetWriterService {
    fn apply_block(&self, block: &Block) -> Result<(), AppError> {
        log_utxo_info!("UtxoSetWriter.apply_block() | block: {:?}", &block);

        let to_delete: Vec<TransactionOutPoint> = block
            .get_transactions()
            .iter()
            .flat_map(|tx| {
                tx.get_inputs()
                    .iter()
                    .map(|input| input.get_previous_output().clone())
            })
            .collect();

        let to_insert: Vec<Utxo> = block
            .get_transactions()
            .iter()
            .flat_map(|tx| {
                tx.get_outputs()
                    .iter()
                    .enumerate()
                    .map(move |(index, output)| {
                        let outpoint = TransactionOutPoint::new(tx.get_hash(), index);
                        Ok(Utxo::new(outpoint, output.clone()))
                    })
            })
            .collect::<Result<_, AppError>>()?;

        log_utxo_debug!("UtxoSetWriter.apply_block() | to_delete: {:?}", to_delete);
        log_utxo_debug!("UtxoSetWriter.apply_block() | to_insert: {:?}", to_insert);

        let utxo_repo = self.utxo_repo.clone();

        let unit_of_work = self.utxo_repo.get_utxo_set_append_block_unit_of_work();
        unit_of_work.run_in_transaction(Box::new(move |ctx| {
            for outpoint in &to_delete {
                utxo_repo.delete_utxo(Some(ctx), outpoint)?;
            }
            for utxo in &to_insert {
                utxo_repo.insert_utxo(Some(ctx), utxo.clone())?;
            }
            Ok(AtomicTransactionOutput::new(()))
        }))?;

        log_utxo_info!(
            "UtxoSetWriter.apply_block(): UtxoSet successfully updated for block ({}) ",
            block.get_hash()
        );
        Ok(())
    }
}
