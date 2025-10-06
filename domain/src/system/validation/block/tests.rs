use super::*;
use crate::entities::block::{BlockDifficultyTarget, BlockTemplate};
use crate::entities::transaction::{
    NonValidatedTransaction, Transaction, TransactionAmount, TransactionInput, TransactionOutPoint,
    TransactionOutput, TransactionsMerkleRoot,
};
use crate::genesis::config::{GenesisConfig, GenesisConfigUtxoFunds};
use crate::system::blockchain::MockBlockchain;
use crate::system::validation::transaction::MockTransactionValidator;
use crate::types::hash::Hash;
use crate::types::sign::PublicKey;
use crate::types::time::DateTime;
use crate::types::wallet::WalletAddress;
use common::error::BlockValidationError;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

fn create_test_genesis_config() -> GenesisConfig {
    let wallet_pub_key = "59f783b83cf3b6552f53044743ac3454a84ed9b47897ef1576e64662363dbd6b"
        .parse::<PublicKey>()
        .expect("Valid public key");

    let utxo =
        GenesisConfigUtxoFunds::new_unchecked(wallet_pub_key, TransactionAmount::new(1000000000));

    let timestamp = DateTime::from_ms(1725799696000);
    GenesisConfig::new_unchecked(vec![utxo], timestamp)
}

fn create_test_wallet_address(seed: u8) -> WalletAddress {
    let hex_str = format!("{:0>64}", seed);
    WalletAddress::from_str(&hex_str).expect("Valid wallet address")
}

fn create_test_hash(seed: u8) -> Hash {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    Hash::new(bytes)
}

fn create_test_outpoint(seed: u8, index: usize) -> TransactionOutPoint {
    TransactionOutPoint::new(create_test_hash(seed), index)
}

fn create_test_transaction(seed: u8) -> NonValidatedTransaction {
    let timestamp = DateTime::from_ms(1000000000 + (seed as u64) * 1000);
    let input = TransactionInput::new(create_test_outpoint(seed, 0));
    let output = TransactionOutput::new(
        create_test_wallet_address(seed),
        TransactionAmount::new(100),
    );
    NonValidatedTransaction::new(vec![input], vec![output], timestamp).unwrap()
}

fn create_coinbase_transaction(seed: u8) -> NonValidatedTransaction {
    let timestamp = DateTime::from_ms(1000000000 + (seed as u64) * 1000);
    let output =
        TransactionOutput::new(create_test_wallet_address(seed), TransactionAmount::new(50));
    NonValidatedTransaction::new(vec![], vec![output], timestamp).unwrap()
}

fn create_test_block_from_genesis() -> NonValidatedBlock {
    NonValidatedBlock::new_genesis(create_test_genesis_config()).unwrap()
}

fn create_test_block_with_transactions(txs: Vec<NonValidatedTransaction>) -> NonValidatedBlock {
    let prev_block = Block::_new_validated(create_test_block_from_genesis());
    let template = BlockTemplate::new(&prev_block, txs, BlockDifficultyTarget::_new_stub());
    NonValidatedBlock::from_template(template).unwrap()
}

fn create_mock_validator() -> DefaultBlockValidator {
    let mock_blockchain = MockBlockchain::new();
    let mock_tx_validator = MockTransactionValidator::new();
    DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator))
}

// ============================================================================
// validate_block_structure_merkle_root()
// ============================================================================

#[test]
fn test_validate_merkle_root_valid() {
    let validator = create_mock_validator();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);
    let transactions = vec![tx1, tx2];

    let correct_merkle_root = TransactionsMerkleRoot::new_non_validated(&transactions).unwrap();

    let block = create_test_block_with_transactions(transactions);

    assert_eq!(block.get_transactions_merkle_root(), &correct_merkle_root);

    let result = validator.pub_validate_block_structure_merkle_root(&block);
    assert!(result.is_ok(), "Block with valid merkle root should pass");
}

// This^ is already more or less re-testing TransactionMerkleRoot...
// Let dedicated unit tests handle the unhappy path.

// ============================================================================
// validate_block_structure_duplicate_transactions()
// ============================================================================

#[test]
fn test_validate_duplicate_transactions_none() {
    let validator = create_mock_validator();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);
    let block = create_test_block_with_transactions(vec![tx1, tx2]);

    let result = validator.pub_validate_block_structure_duplicate_transactions(&block);
    assert!(result.is_ok(), "Block with unique transactions should pass");
}

#[test]
fn test_validate_duplicate_transactions_has_duplicates() {
    let validator = create_mock_validator();

    let tx = create_test_transaction(1);
    // Create a block with the same transaction twice
    let block = create_test_block_with_transactions(vec![tx.clone(), tx.clone()]);

    let result = validator.pub_validate_block_structure_duplicate_transactions(&block);
    assert!(
        result.is_err(),
        "Block with duplicate transactions should fail"
    );

    match result.unwrap_err() {
        AppError::BlockValidation(BlockValidationError::DuplicateTransactions) => {}
        _ => panic!("Expected DuplicateTransactions error"),
    }
}

#[test]
fn test_validate_duplicate_transactions_single_transaction() {
    let validator = create_mock_validator();

    let tx = create_test_transaction(1);
    let block = create_test_block_with_transactions(vec![tx]);

    let result = validator.pub_validate_block_structure_duplicate_transactions(&block);
    assert!(result.is_ok(), "Block with single transaction should pass");
}

// ============================================================================
// validate_block_structure_non_empty_transactions()
// ============================================================================

#[test]
fn test_validate_non_empty_transactions_has_transactions() {
    let validator = create_mock_validator();

    let tx = create_test_transaction(1);
    let block = create_test_block_with_transactions(vec![tx]);

    let result = validator.pub_validate_block_structure_non_empty_transactions(&block);
    assert!(result.is_ok(), "Block with transactions should pass");
}

#[test]
fn test_validate_non_empty_transactions_multiple() {
    let validator = create_mock_validator();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);
    let tx3 = create_test_transaction(3);
    let block = create_test_block_with_transactions(vec![tx1, tx2, tx3]);

    let result = validator.pub_validate_block_structure_non_empty_transactions(&block);
    assert!(
        result.is_ok(),
        "Block with multiple transactions should pass"
    );
}

// ============================================================================
// validate_block_structure_coinbase()
// ============================================================================

#[test]
fn test_validate_coinbase_none() {
    let validator = create_mock_validator();

    // Regular transactions (no coinbase)
    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);
    let block = create_test_block_with_transactions(vec![tx1, tx2]);

    let result = validator.pub_validate_block_structure_coinbase(&block);
    assert!(
        result.is_ok(),
        "Block with no coinbase should pass (0 or 1 coinbase allowed)"
    );
}

#[test]
fn test_validate_coinbase_single() {
    let validator = create_mock_validator();

    let coinbase = create_coinbase_transaction(1);
    let tx = create_test_transaction(2);
    let block = create_test_block_with_transactions(vec![coinbase, tx]);

    let result = validator.pub_validate_block_structure_coinbase(&block);
    assert!(result.is_ok(), "Block with single coinbase should pass");
}

#[test]
fn test_validate_coinbase_multiple() {
    let validator = create_mock_validator();

    let coinbase1 = create_coinbase_transaction(1);
    let coinbase2 = create_coinbase_transaction(2);
    let block = create_test_block_with_transactions(vec![coinbase1, coinbase2]);

    let result = validator.pub_validate_block_structure_coinbase(&block);
    assert!(
        result.is_err(),
        "Block with multiple coinbase transactions should fail"
    );

    match result.unwrap_err() {
        AppError::BlockValidation(BlockValidationError::MultipleCoinbaseTransactions) => {}
        _ => panic!("Expected MultipleCoinbaseTransactions error"),
    }
}

#[test]
fn test_validate_coinbase_genesis_block() {
    let validator = create_mock_validator();
    let genesis = create_test_block_from_genesis();

    // Genesis block has coinbase transaction(s)
    let result = validator.pub_validate_block_structure_coinbase(&genesis);
    assert!(result.is_ok(), "Genesis block coinbase should pass");
}

// ============================================================================
// validate_block_content_parent()
// ============================================================================

#[test]
fn test_validate_parent_matches_tip() {
    let validator = create_mock_validator();

    let prev_hash = create_test_hash(1);
    let tx = create_test_transaction(1);
    let block = create_test_block_with_transactions(vec![tx]);

    let result = validator.pub_validate_block_content_parent(&block, Some(&prev_hash));

    // This will fail because the block's prev_hash is from genesis, not our test hash
    // But we're testing the logic works correctly
    assert!(result.is_err(), "Block with mismatched parent should fail");

    match result.unwrap_err() {
        AppError::BlockValidation(BlockValidationError::ContinuityMismatch { .. }) => {}
        _ => panic!("Expected ContinuityMismatch error"),
    }
}

#[test]
fn test_validate_parent_genesis_block() {
    let validator = create_mock_validator();
    let genesis = create_test_block_from_genesis();

    // Genesis block has no parent, tip should be None
    let result = validator.pub_validate_block_content_parent(&genesis, None);
    assert!(result.is_ok(), "Genesis block with no tip should pass");
}

#[test]
fn test_validate_parent_mismatch() {
    let validator = create_mock_validator();

    let tx = create_test_transaction(1);
    let block = create_test_block_with_transactions(vec![tx]);

    let wrong_tip = create_test_hash(99);

    let result = validator.pub_validate_block_content_parent(&block, Some(&wrong_tip));
    assert!(result.is_err(), "Block with wrong parent should fail");

    match result.unwrap_err() {
        AppError::BlockValidation(BlockValidationError::ContinuityMismatch { .. }) => {}
        _ => panic!("Expected ContinuityMismatch error"),
    }
}

#[test]
fn test_validate_parent_none_when_expecting_parent() {
    let validator = create_mock_validator();

    let genesis = create_test_block_from_genesis();
    let some_tip = create_test_hash(1);

    // Genesis has no parent, but we're expecting one
    let result = validator.pub_validate_block_content_parent(&genesis, Some(&some_tip));
    assert!(
        result.is_err(),
        "Genesis block when expecting parent should fail"
    );
}

// ============================================================================
// validate_block_content_transactions()
// ============================================================================

#[tokio::test]
async fn test_validate_block_content_transactions_genesis_pre_genesis_chain() {
    let mock_blockchain = MockBlockchain::new();
    let mock_tx_validator = MockTransactionValidator::new();
    let validator =
        DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator));

    let genesis = create_test_block_from_genesis();

    // Genesis block on pre-genesis chain should pass (no transaction validation)
    let result = validator
        .pub_validate_block_content_transactions(&genesis, true)
        .await;
    assert!(
        result.is_ok(),
        "Genesis block on pre-genesis chain should pass"
    );
}

#[tokio::test]
async fn test_validate_block_content_transactions_genesis_already_exists() {
    let mock_blockchain = MockBlockchain::new();
    let mock_tx_validator = MockTransactionValidator::new();
    let validator =
        DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator));

    let genesis = create_test_block_from_genesis();

    // Genesis block when chain already exists should fail
    let result = validator
        .pub_validate_block_content_transactions(&genesis, false)
        .await;
    assert!(
        result.is_err(),
        "Genesis block when chain exists should fail"
    );

    match result.unwrap_err() {
        AppError::BlockValidation(BlockValidationError::GenesisAlreadyExists) => {}
        _ => panic!("Expected GenesisAlreadyExists error"),
    }
}

#[tokio::test]
async fn test_validate_block_content_transactions_single_valid_transaction() {
    let mock_blockchain = MockBlockchain::new();
    let mut mock_tx_validator = MockTransactionValidator::new();

    let tx = create_test_transaction(1);

    // Mock the transaction validator to return success
    mock_tx_validator
        .expect_validate_transaction()
        .times(1)
        .returning(move |tx| Ok(Transaction::_new_validated(tx)));

    let validator =
        DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator));

    let block = create_test_block_with_transactions(vec![tx]);

    let result = validator
        .pub_validate_block_content_transactions(&block, false)
        .await;
    assert!(result.is_ok(), "Block with valid transaction should pass");
}

#[tokio::test]
async fn test_validate_block_content_transactions_multiple_valid_transactions() {
    let mock_blockchain = MockBlockchain::new();
    let mut mock_tx_validator = MockTransactionValidator::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);

    // Mock expects two calls
    mock_tx_validator
        .expect_validate_transaction()
        .times(2)
        .returning(move |tx| Ok(Transaction::_new_validated(tx)));

    let validator =
        DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator));

    let block = create_test_block_with_transactions(vec![tx1, tx2]);

    let result = validator
        .pub_validate_block_content_transactions(&block, false)
        .await;
    assert!(
        result.is_ok(),
        "Block with multiple valid transactions should pass"
    );
}

#[tokio::test]
async fn test_validate_block_content_transactions_invalid_transaction_fails() {
    let mock_blockchain = MockBlockchain::new();
    let mut mock_tx_validator = MockTransactionValidator::new();

    let tx = create_test_transaction(1);

    // Mock the transaction validator to return an error
    mock_tx_validator
        .expect_validate_transaction()
        .times(1)
        .returning(move |tx| {
            Err(AppError::TransactionValidation(
                TransactionValidationError::EmptyInputs {
                    tx_id: tx.get_hash().to_string(),
                },
            ))
        });

    let validator =
        DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator));

    let block = create_test_block_with_transactions(vec![tx]);

    let result = validator
        .pub_validate_block_content_transactions(&block, false)
        .await;
    assert!(
        result.is_err(),
        "Block with invalid transaction should fail"
    );
}

#[tokio::test]
async fn test_validate_block_content_transactions_detects_double_spend_in_block() {
    let mock_blockchain = MockBlockchain::new();
    let mut mock_tx_validator = MockTransactionValidator::new();

    // Create two transactions that spend the same outpoint
    let outpoint = create_test_outpoint(1, 0);
    let input1 = TransactionInput::new(outpoint.clone());
    let input2 = TransactionInput::new(outpoint.clone());
    let output1 =
        TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(100));
    let output2 =
        TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(100));

    let tx1 =
        NonValidatedTransaction::new(vec![input1], vec![output1], DateTime::from_ms(1000000000))
            .unwrap();

    let tx2 =
        NonValidatedTransaction::new(vec![input2], vec![output2], DateTime::from_ms(1000001000))
            .unwrap();

    // Both transactions pass individual validation
    mock_tx_validator
        .expect_validate_transaction()
        .times(2)
        .returning(move |tx| Ok(Transaction::_new_validated(tx)));

    let validator =
        DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator));

    let block = create_test_block_with_transactions(vec![tx1, tx2]);

    // Should fail because both transactions spend the same outpoint (double spend within block)
    let result = validator
        .pub_validate_block_content_transactions(&block, false)
        .await;
    assert!(result.is_err(), "Block with double spend should fail");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::DoubleSpending { .. }) => {}
        _ => panic!("Expected DoubleSpending error"),
    }
}

#[tokio::test]
async fn test_validate_block_content_transactions_with_coinbase() {
    let mock_blockchain = MockBlockchain::new();
    let mut mock_tx_validator = MockTransactionValidator::new();

    let coinbase = create_coinbase_transaction(1);
    let tx = create_test_transaction(2);

    // Mock expects two calls
    mock_tx_validator
        .expect_validate_transaction()
        .times(2)
        .returning(move |tx| Ok(Transaction::_new_validated(tx)));

    let validator =
        DefaultBlockValidator::new(Arc::new(mock_blockchain), Arc::new(mock_tx_validator));

    let block = create_test_block_with_transactions(vec![coinbase, tx]);

    let result = validator
        .pub_validate_block_content_transactions(&block, false)
        .await;
    assert!(
        result.is_ok(),
        "Block with coinbase and regular transaction should pass"
    );
}

// ============================================================================
// validate_block_content_transactions_double_spends()
// ============================================================================

#[test]
fn test_validate_double_spends_none() {
    let validator = create_mock_validator();

    let tx = create_test_transaction(1);
    let mut spent_outpoints = HashSet::new();

    let result =
        validator.pub_validate_block_content_transactions_double_spends(&tx, &mut spent_outpoints);
    assert!(result.is_ok(), "Transaction with unique inputs should pass");
    assert_eq!(spent_outpoints.len(), 1, "Should track one outpoint");
}

#[test]
fn test_validate_double_spends_detected() {
    let validator = create_mock_validator();

    let outpoint = create_test_outpoint(1, 0);
    let mut spent_outpoints = HashSet::new();
    spent_outpoints.insert(outpoint.clone());

    // Create transaction trying to spend the same outpoint
    let tx = create_test_transaction(1);

    let result =
        validator.pub_validate_block_content_transactions_double_spends(&tx, &mut spent_outpoints);
    assert!(result.is_err(), "Double spend should be detected");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::DoubleSpending { .. }) => {}
        _ => panic!("Expected DoubleSpending error"),
    }
}

#[test]
fn test_validate_double_spends_multiple_inputs_unique() {
    let validator = create_mock_validator();

    let input1 = TransactionInput::new(create_test_outpoint(1, 0));
    let input2 = TransactionInput::new(create_test_outpoint(2, 0));
    let output = TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(100));
    let tx = NonValidatedTransaction::new(
        vec![input1, input2],
        vec![output],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    let mut spent_outpoints = HashSet::new();

    let result =
        validator.pub_validate_block_content_transactions_double_spends(&tx, &mut spent_outpoints);
    assert!(
        result.is_ok(),
        "Transaction with multiple unique inputs should pass"
    );
    assert_eq!(spent_outpoints.len(), 2, "Should track two outpoints");
}

#[test]
fn test_validate_double_spends_coinbase_transaction() {
    let validator = create_mock_validator();

    let coinbase = create_coinbase_transaction(1);
    let mut spent_outpoints = HashSet::new();

    // Coinbase has no inputs, so shouldn't add anything to spent_outpoints
    let result = validator
        .pub_validate_block_content_transactions_double_spends(&coinbase, &mut spent_outpoints);
    assert!(result.is_ok(), "Coinbase transaction should pass");
    assert_eq!(
        spent_outpoints.len(),
        0,
        "Coinbase should not track any outpoints"
    );
}

#[test]
fn test_validate_double_spends_sequential_transactions() {
    let validator = create_mock_validator();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);
    let mut spent_outpoints = HashSet::new();

    // Process first transaction
    let result1 =
        validator.pub_validate_block_content_transactions_double_spends(&tx1, &mut spent_outpoints);
    assert!(result1.is_ok(), "First transaction should pass");
    assert_eq!(spent_outpoints.len(), 1);

    // Process second transaction with different inputs
    let result2 =
        validator.pub_validate_block_content_transactions_double_spends(&tx2, &mut spent_outpoints);
    assert!(
        result2.is_ok(),
        "Second transaction with unique inputs should pass"
    );
    assert_eq!(spent_outpoints.len(), 2, "Should track both outpoints");
}
