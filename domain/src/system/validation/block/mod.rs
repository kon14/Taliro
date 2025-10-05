use super::transaction::TransactionValidator;
use crate::entities::block::{Block, NonValidatedBlock};
use crate::entities::transaction::{
    NonValidatedTransaction, TransactionOutPoint, TransactionsMerkleRoot,
};
use crate::system::blockchain::Blockchain;
use crate::types::hash::Hash;
use async_trait::async_trait;
use common::error::{AppError, BlockValidationError, TransactionValidationError};
use common::log_blk_info;
use std::collections::HashSet;
use std::sync::Arc;

/// Ensures blocks are validated against their structure and the current state of the blockchain.
#[async_trait]
pub(crate) trait BlockValidator: Send + Sync + std::fmt::Debug {
    /// Performs full structural and content level validation on a block.
    async fn validate_block(&self, block: NonValidatedBlock) -> Result<Block, AppError>;

    /// Performs structural validation on a block.<br />
    /// These checks feature <strong>no dependencies on external state</strong>.<br />
    /// Results are deterministic and reproducible.
    fn validate_block_structure(&self, block: &NonValidatedBlock) -> Result<(), AppError>;

    /// Performs content validation on a block.<br />
    /// These checks <strong>depend on external state</strong>.<br />
    /// Results may vary based on the current state of the blockchain.
    async fn validate_block_content(&self, block: &NonValidatedBlock) -> Result<(), AppError>;
}

#[derive(Debug)]
pub(crate) struct DefaultBlockValidator {
    blockchain: Arc<dyn Blockchain>,
    tx_validator: Arc<dyn TransactionValidator>,
}

#[async_trait]
impl BlockValidator for DefaultBlockValidator {
    async fn validate_block(&self, block: NonValidatedBlock) -> Result<Block, AppError> {
        log_blk_info!("BlockValidator.validate_block() | block: {:?}", &block);

        let block_hash = block.get_hash();

        // Early idempotency check.
        // Prevents non-canon block race conditions.
        let block_known = self.blockchain.has_known_block(&block_hash).await?;
        if block_known {
            return Err(AppError::BlockValidation(
                BlockValidationError::BlockAlreadyKnown {
                    hash: block.get_hash().to_string(),
                },
            ));
        }

        self.validate_block_structure(&block)?;
        self.validate_block_content(&block).await?;

        log_blk_info!(
            "BlockValidator.validate_block(): Successfully validated block ({}) ",
            block_hash
        );
        let validated_block = Block::_new_validated(block);
        Ok(validated_block)
    }

    fn validate_block_structure(&self, block: &NonValidatedBlock) -> Result<(), AppError> {
        Self::validate_block_structure_header(block)?;
        Self::validate_block_structure_merkle_root(block)?;
        Self::validate_block_structure_duplicate_transactions(block)?;
        Self::validate_block_structure_non_empty_transactions(block)?;
        Self::validate_block_structure_coinbase(block)?;
        Ok(())
    }

    async fn validate_block_content(&self, block: &NonValidatedBlock) -> Result<(), AppError> {
        let tip_info = self.blockchain.get_tip_info().await?;
        self.validate_block_content_parent(block, tip_info.as_ref().map(|info| &info.0))?;
        self.validate_block_content_consensus(block)?;
        self.validate_block_content_transactions(block, tip_info.is_none())
            .await?;
        Ok(())
    }
}

impl DefaultBlockValidator {
    pub(crate) fn new(
        blockchain: Arc<dyn Blockchain>,
        tx_validator: Arc<dyn TransactionValidator>,
    ) -> Self {
        Self {
            blockchain,
            tx_validator,
        }
    }

    #[allow(unused)]
    fn validate_block_structure_header(block: &NonValidatedBlock) -> Result<(), AppError> {
        // TODO:
        // Is the timestamp a sane value (not too far in the future/past)?
        Ok(())
    }

    fn validate_block_structure_merkle_root(block: &NonValidatedBlock) -> Result<(), AppError> {
        let expected_root = block.get_transactions_merkle_root();
        let computed_root = TransactionsMerkleRoot::new_non_validated(block.get_transactions())?;

        if expected_root != &computed_root {
            return Err(AppError::BlockValidation(
                BlockValidationError::InvalidMerkleRoot {
                    expected: expected_root.inner().to_string(),
                    actual: computed_root.inner().to_string(),
                },
            ));
        }
        Ok(())
    }

    fn validate_block_structure_duplicate_transactions(
        block: &NonValidatedBlock,
    ) -> Result<(), AppError> {
        let tx_hash_ids = block
            .get_transactions()
            .iter()
            .map(|tx| tx.get_hash())
            .collect::<HashSet<_>>();
        if tx_hash_ids.len() != block.get_transactions().len() {
            Err(AppError::BlockValidation(
                BlockValidationError::DuplicateTransactions,
            ))
        } else {
            Ok(())
        }
    }

    fn validate_block_structure_non_empty_transactions(
        block: &NonValidatedBlock,
    ) -> Result<(), AppError> {
        if block.get_transactions().is_empty() {
            return Err(AppError::BlockValidation(
                BlockValidationError::NoTransactions,
            ));
        }
        Ok(())
    }

    fn validate_block_structure_coinbase(block: &NonValidatedBlock) -> Result<(), AppError> {
        let coinbase_tx = block
            .get_transactions()
            .iter()
            .filter(|tx| tx.is_coinbase_tx())
            .collect::<Vec<_>>();
        if coinbase_tx.len() > 1 {
            return Err(AppError::BlockValidation(
                BlockValidationError::MultipleCoinbaseTransactions,
            ));
        }
        // TODO: confirm coinbase value and maturity
        Ok(())
    }

    fn validate_block_content_parent(
        &self,
        block: &NonValidatedBlock,
        local_tip_hash: Option<&Hash>,
    ) -> Result<(), AppError> {
        if block.get_prev_block_hash().as_ref() != local_tip_hash {
            return Err(AppError::BlockValidation(
                BlockValidationError::ContinuityMismatch {
                    block_prev_hash: block.get_prev_block_hash().map(|h| h.to_string()),
                    blockchain_tip_hash: local_tip_hash.map(|tip_info| tip_info.to_string()),
                },
            ));
        }
        Ok(())
    }

    #[allow(unused)]
    fn validate_block_content_consensus(&self, block: &NonValidatedBlock) -> Result<(), AppError> {
        // TODO:
        // Proof of Work / Proof of Stake / Quorum is valid
        // Correct difficulty level
        Ok(())
    }

    async fn validate_block_content_transactions(
        &self,
        block: &NonValidatedBlock,
        pre_genesis_chain: bool,
    ) -> Result<(), AppError> {
        if block.is_genesis_block() {
            if pre_genesis_chain {
                return Ok(());
            }
            return Err(AppError::BlockValidation(
                BlockValidationError::GenesisAlreadyExists,
            ));
        }

        // Rolling with readability over performance optimizations...
        let mut spent_outpoints = HashSet::new();
        for tx in block.get_transactions() {
            self.tx_validator.validate_transaction(tx.clone()).await?;
            self.validate_block_content_transactions_double_spends(tx, &mut spent_outpoints)?;
            // TODO: signature verification
        }
        Ok(())
    }

    fn validate_block_content_transactions_double_spends(
        &self,
        tx: &NonValidatedTransaction,
        spent_outpoints: &mut HashSet<TransactionOutPoint>,
    ) -> Result<(), AppError> {
        for txin in tx.get_inputs() {
            let outpoint = txin.get_previous_output();
            if !spent_outpoints.insert(outpoint.clone()) {
                return Err(AppError::TransactionValidation(
                    TransactionValidationError::DoubleSpending {
                        tx_id: tx.get_hash().to_string(),
                        outpoint: outpoint.to_string(),
                    },
                ));
            }
        }
        Ok(())
    }
}
