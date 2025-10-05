use super::*;
use crate::entities::block::Block;
use crate::entities::transaction::{Transaction, TransactionAmount};
use crate::genesis::config::{GenesisConfig, GenesisConfigUtxoFunds};
use crate::types::hash::Hash;
use crate::types::sign::PublicKey;
use crate::types::time::DateTime;

fn create_test_genesis_config() -> GenesisConfig {
    let wallet_pub_key = "59f783b83cf3b6552f53044743ac3454a84ed9b47897ef1576e64662363dbd6b"
        .parse::<PublicKey>()
        .expect("Valid public key");

    let utxo =
        GenesisConfigUtxoFunds::new_unchecked(wallet_pub_key, TransactionAmount::new(1000000000));

    // 2025-09-08T12:34:56Z as Unix timestamp in milliseconds
    let timestamp = DateTime::from_ms(1725799696000);

    GenesisConfig::new_unchecked(vec![utxo], timestamp)
}

fn create_test_block_with_transactions(txs: Vec<Transaction>) -> Block {
    use crate::entities::block::{BlockDifficultyTarget, BlockTemplate, NonValidatedBlock};
    use crate::entities::transaction::NonValidatedTransaction;

    // First create a genesis block to use as the previous block for the template
    let prev_block = Block::_new_validated(
        NonValidatedBlock::new_genesis(create_test_genesis_config()).unwrap(),
    );

    // Convert validated transactions to non-validated for block creation
    let non_validated_txs: Vec<NonValidatedTransaction> =
        txs.into_iter().map(|tx| tx.invalidate()).collect();

    // Create a block template with the provided transactions
    let template = BlockTemplate::new(
        &prev_block,
        non_validated_txs,
        BlockDifficultyTarget::_new_stub(),
    );

    // Create the block from the template
    let non_validated_block = NonValidatedBlock::from_template(template).unwrap();
    Block::_new_validated(non_validated_block)
}

fn create_mock_transaction(seed: u8) -> Transaction {
    use crate::entities::transaction::NonValidatedTransaction;

    // Create a unique timestamp based on the seed to ensure different hashes
    let timestamp = DateTime::from_ms(1000000000 + (seed as u64) * 1000);
    let tx = NonValidatedTransaction::new(vec![], vec![], timestamp).unwrap();
    Transaction::_new_validated(tx)
}

#[tokio::test]
async fn test_new_mempool_is_empty() {
    let mempool = DefaultMempool::new();

    let fake_hash = Hash::new([1u8; 32]);
    let result = mempool.get_transaction(&fake_hash).await;

    assert!(result.is_none(), "New mempool should be empty");
}

#[tokio::test]
async fn test_add_transaction() {
    let mempool = DefaultMempool::new();
    let tx = create_mock_transaction(1);
    let tx_hash = tx.get_hash();

    mempool.add_transaction(tx.clone()).await.unwrap();

    let retrieved = mempool.get_transaction(&tx_hash).await;
    assert!(
        retrieved.is_some(),
        "Transaction should be added to mempool"
    );
    assert_eq!(
        retrieved.unwrap().get_hash(),
        tx_hash,
        "Retrieved transaction should match"
    );
}

#[tokio::test]
async fn test_add_multiple_transactions() {
    let mempool = DefaultMempool::new();
    let tx1 = create_mock_transaction(1);
    let tx2 = create_mock_transaction(2);
    let tx3 = create_mock_transaction(3);

    let hash1 = tx1.get_hash();
    let hash2 = tx2.get_hash();
    let hash3 = tx3.get_hash();

    mempool.add_transaction(tx1).await.unwrap();
    mempool.add_transaction(tx2).await.unwrap();
    mempool.add_transaction(tx3).await.unwrap();

    assert!(
        mempool.get_transaction(&hash1).await.is_some(),
        "Transaction 1 should exist"
    );
    assert!(
        mempool.get_transaction(&hash2).await.is_some(),
        "Transaction 2 should exist"
    );
    assert!(
        mempool.get_transaction(&hash3).await.is_some(),
        "Transaction 3 should exist"
    );
}

#[tokio::test]
async fn test_get_nonexistent_transaction() {
    let mempool = DefaultMempool::new();
    let fake_hash = Hash::new([99u8; 32]);

    let result = mempool.get_transaction(&fake_hash).await;

    assert!(
        result.is_none(),
        "Should return None for non-existent transaction"
    );
}

#[tokio::test]
async fn test_add_duplicate_transaction_overwrites() {
    let mempool = DefaultMempool::new();
    let tx = create_mock_transaction(1);
    let tx_hash = tx.get_hash();

    // Add the same transaction twice
    mempool.add_transaction(tx.clone()).await.unwrap();
    mempool.add_transaction(tx.clone()).await.unwrap();

    let retrieved = mempool.get_transaction(&tx_hash).await;
    assert!(
        retrieved.is_some(),
        "Transaction should still exist after duplicate add"
    );
}

#[tokio::test]
async fn test_apply_block_removes_transactions() {
    let mempool = DefaultMempool::new();

    // Create transactions and add them to mempool
    let tx1 = create_mock_transaction(1);
    let tx2 = create_mock_transaction(2);
    let tx3 = create_mock_transaction(3);

    let hash1 = tx1.get_hash();
    let hash2 = tx2.get_hash();
    let hash3 = tx3.get_hash();

    mempool.add_transaction(tx1.clone()).await.unwrap();
    mempool.add_transaction(tx2.clone()).await.unwrap();
    mempool.add_transaction(tx3.clone()).await.unwrap();

    // Create a block containing tx1 and tx2
    let block = create_test_block_with_transactions(vec![tx1, tx2]);

    // Debug: Check what transactions are actually in the block
    println!("Block has {} transactions", block.get_transactions().len());
    for (i, tx) in block.get_transactions().iter().enumerate() {
        println!("Block tx {}: hash = {}", i, tx.get_hash());
    }
    println!("Expected hash1: {}", hash1);
    println!("Expected hash2: {}", hash2);
    println!("Expected hash3: {}", hash3);

    // Apply block to mempool
    mempool.apply_block(&block).await.unwrap();

    // tx1 and tx2 should be removed, tx3 should remain
    let tx1_in_pool = mempool.get_transaction(&hash1).await.is_some();
    let tx2_in_pool = mempool.get_transaction(&hash2).await.is_some();
    let tx3_in_pool = mempool.get_transaction(&hash3).await.is_some();

    println!("After apply_block:");
    println!("  tx1 in pool: {}", tx1_in_pool);
    println!("  tx2 in pool: {}", tx2_in_pool);
    println!("  tx3 in pool: {}", tx3_in_pool);

    assert!(!tx1_in_pool, "tx1 should be removed");
    assert!(!tx2_in_pool, "tx2 should be removed");
    assert!(tx3_in_pool, "tx3 should remain");
}

#[tokio::test]
async fn test_apply_block_with_no_matching_transactions() {
    let mempool = DefaultMempool::new();

    // Add a transaction to mempool
    let tx_in_mempool = create_mock_transaction(1);
    let hash_in_mempool = tx_in_mempool.get_hash();
    mempool.add_transaction(tx_in_mempool).await.unwrap();

    // Create a block with different transactions
    let tx_in_block = create_mock_transaction(2);
    let block = create_test_block_with_transactions(vec![tx_in_block]);

    // Apply block
    mempool.apply_block(&block).await.unwrap();

    // Original transaction should still be in mempool
    assert!(
        mempool.get_transaction(&hash_in_mempool).await.is_some(),
        "Unrelated transaction should remain in mempool"
    );
}

#[tokio::test]
async fn test_rm_transaction_removes_and_returns() {
    let mempool = DefaultMempool::new();
    let tx = create_mock_transaction(1);
    let tx_hash = tx.get_hash();

    mempool.add_transaction(tx.clone()).await.unwrap();

    // Remove transaction
    let removed = mempool.rm_transaction(&tx_hash).await;

    assert!(removed.is_some(), "Should return removed transaction");
    assert_eq!(
        removed.unwrap().get_hash(),
        tx_hash,
        "Removed transaction should match"
    );
    assert!(
        mempool.get_transaction(&tx_hash).await.is_none(),
        "Transaction should be removed"
    );
}

#[tokio::test]
async fn test_rm_nonexistent_transaction() {
    let mempool = DefaultMempool::new();
    let fake_hash = Hash::new([99u8; 32]);

    let removed = mempool.rm_transaction(&fake_hash).await;

    assert!(
        removed.is_none(),
        "Should return None when removing non-existent transaction"
    );
}

#[tokio::test]
async fn test_concurrent_add_and_get() {
    use tokio::task;

    let mempool = std::sync::Arc::new(DefaultMempool::new());
    let mut handles = vec![];

    // Spawn multiple tasks adding transactions
    for i in 0..10 {
        let mempool_clone = mempool.clone();
        let handle = task::spawn(async move {
            let tx = create_mock_transaction(i);
            mempool_clone.add_transaction(tx).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify we can still read from mempool (basic concurrency test)
    let fake_hash = Hash::new([1u8; 32]);
    let _ = mempool.get_transaction(&fake_hash).await;
}

#[tokio::test]
async fn test_apply_block_to_empty_mempool() {
    let mempool = DefaultMempool::new();

    // Create a block with transactions
    let tx = create_mock_transaction(1);
    let block = create_test_block_with_transactions(vec![tx]);

    // Apply block to empty mempool (should not error)
    let result = mempool.apply_block(&block).await;

    assert!(
        result.is_ok(),
        "Applying block to empty mempool should succeed"
    );
}
