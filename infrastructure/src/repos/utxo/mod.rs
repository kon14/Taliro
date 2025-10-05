use crate::ext::{AppErrorExtInfrastructure, TransactionContextExtInfrastructure};
use crate::storage::SledStorage;
use crate::tx::SledUnitOfWork;
use crate::tx::trees::{SledTxTrees, SledTxUtxoSetAppendBlockTrees};
use common::error::AppError;
use common::tx::UnitOfWork;
use common::tx::ctx::AtomicTransactionContext;
use domain::encode::{TryDecode, TryEncode};
use domain::entities::transaction::{TransactionOutPoint, TransactionOutput, Utxo};
use domain::repos::utxo::UtxoRepository;
use sled::Tree;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct SledUtxoRepository {
    utxo_tree: Tree,
}

impl Debug for SledUtxoRepository {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SledUtxoRepository")
            .field("utxo_tree", &SledStorage::UTXO_TREE)
            .finish()
    }
}

impl SledUtxoRepository {
    pub fn open(utxo_tree: Tree) -> Result<Self, AppError> {
        let repo = Self { utxo_tree };
        Ok(repo)
    }
}

impl UtxoRepository for SledUtxoRepository {
    fn get_utxo_set_append_block_unit_of_work(&self) -> Arc<dyn UnitOfWork> {
        let trees = SledTxUtxoSetAppendBlockTrees {
            utxo_tree: self.utxo_tree.clone(),
        };
        let trees = SledTxTrees::UtxoSetAppendBlock(trees);
        Arc::new(SledUnitOfWork::new(trees))
    }

    fn get_output(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        outpoint: &TransactionOutPoint,
    ) -> Result<Option<TransactionOutput>, AppError> {
        let key = outpoint.try_encode()?;
        let utxo = if let Some(tx_ctx) = tx_ctx {
            let utxo_tree = tx_ctx.get_utxo_tree()?;
            utxo_tree.get(key).to_app_error()?
        } else {
            self.utxo_tree.get(key).to_app_error()?
        };
        if let Some(bytes) = utxo {
            let utxo = TransactionOutput::try_decode(&bytes)?;
            Ok(Some(utxo))
        } else {
            Ok(None)
        }
    }

    fn get_multiple_utxos(&self) -> Result<Vec<Utxo>, AppError> {
        let mut utxos = Vec::new();

        for res in self.utxo_tree.iter() {
            let (key_bytes, value_bytes) = res.map_err(|err| {
                AppError::internal_with_private("Storage read error!", err.to_string())
            })?;
            let outpoint = TransactionOutPoint::try_decode(&key_bytes)?;
            let output = TransactionOutput::try_decode(&value_bytes)?;
            utxos.push(Utxo::new(outpoint, output));
        }

        Ok(utxos)
    }

    fn insert_utxo(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        utxo: Utxo,
    ) -> Result<(), AppError> {
        let key = utxo.get_outpoint().try_encode()?;
        let data = utxo.get_output().try_encode()?;
        if let Some(tx_ctx) = tx_ctx {
            let utxo_tree = tx_ctx.get_utxo_tree()?;
            utxo_tree.insert(key, data).to_app_error()?;
        } else {
            self.utxo_tree.insert(key, data).to_app_error()?;
        }
        Ok(())
    }

    fn delete_utxo(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        outpoint: &TransactionOutPoint,
    ) -> Result<(), AppError> {
        let key = outpoint.try_encode()?;
        if let Some(tx_ctx) = tx_ctx {
            let utxo_tree = tx_ctx.get_utxo_tree()?;
            utxo_tree.remove(key).to_app_error()?;
        } else {
            self.utxo_tree.remove(key).to_app_error()?;
        }
        Ok(())
    }

    fn get_utxo_count(&self) -> usize {
        self.utxo_tree.iter().count()
    }
}
