use super::*;
use crate::entities::block::BlockHeight;
use crate::entities::transaction::TransactionAmount;
use crate::genesis::config::{GenesisConfig, GenesisConfigUtxoFunds};
use crate::repos::blockchain::MockBlockchainRepository;
use crate::repos::outbox::MockOutboxRepository;
use crate::types::hash::Hash;
use crate::types::sign::PublicKey;
use crate::types::time::DateTime;
use std::sync::Arc;

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

#[tokio::test]
async fn test_get_tip_info_with_empty_blockchain() {
    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo.expect_get_tip().returning(|_| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let tip = blockchain.get_tip_info().await.unwrap();

    assert!(tip.is_none(), "Empty blockchain should have no tip");
}

#[tokio::test]
async fn test_get_tip_info_caching() {
    let test_hash = Hash::new([1u8; 32]);
    let test_height = BlockHeight::genesis();

    let mut mock_repo = MockBlockchainRepository::new();

    // First call should fetch from repo (once)
    mock_repo
        .expect_get_tip()
        .times(1)
        .returning(move |_| Ok(Some(test_hash.clone())));

    mock_repo
        .expect_get_height()
        .times(1)
        .returning(move |_, _| Ok(Some(test_height.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    // First call - should fetch from repo
    let tip1 = blockchain.get_tip_info().await.unwrap();
    assert!(tip1.is_some());

    // Second call - should use cache (no additional expectations, test will fail if called again)
    let tip2 = blockchain.get_tip_info().await.unwrap();
    assert_eq!(tip1, tip2);
}

#[tokio::test]
async fn test_get_unknown_block_heights_empty_local_chain() {
    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo.expect_get_tip().returning(|_| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let remote_tip_hash = Hash::new([1u8; 32]);
    let remote_tip_height = BlockHeight::from(5);

    let range = blockchain
        .get_unknown_block_heights((remote_tip_hash, remote_tip_height.clone()))
        .await
        .unwrap();

    assert_eq!(range, Some(BlockHeight::genesis()..=remote_tip_height));
}

#[tokio::test]
async fn test_get_unknown_block_heights_local_behind() {
    let local_hash = Hash::new([2u8; 32]);
    let local_height = BlockHeight::from(3);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(local_hash.clone())));
    mock_repo
        .expect_get_height()
        .returning(move |_, _| Ok(Some(local_height.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let remote_tip_hash = Hash::new([1u8; 32]);
    let remote_tip_height = BlockHeight::from(7);

    let range = blockchain
        .get_unknown_block_heights((remote_tip_hash, remote_tip_height.clone()))
        .await
        .unwrap();

    assert_eq!(range, Some(BlockHeight::from(4)..=remote_tip_height));
}

#[tokio::test]
async fn test_get_unknown_block_heights_local_up_to_date() {
    let local_hash = Hash::new([2u8; 32]);
    let local_height = BlockHeight::from(5);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(local_hash.clone())));
    mock_repo
        .expect_get_height()
        .returning(move |_, _| Ok(Some(local_height.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let remote_tip_hash = Hash::new([1u8; 32]);
    let remote_tip_height = BlockHeight::from(5);

    let range = blockchain
        .get_unknown_block_heights((remote_tip_hash, remote_tip_height))
        .await
        .unwrap();

    assert!(
        range.is_none(),
        "Should return None when local is up-to-date"
    );
}

#[tokio::test]
async fn test_get_unknown_block_heights_local_ahead() {
    let local_hash = Hash::new([2u8; 32]);
    let local_height = BlockHeight::from(10);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(local_hash.clone())));
    mock_repo
        .expect_get_height()
        .returning(move |_, _| Ok(Some(local_height.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let remote_tip_hash = Hash::new([1u8; 32]);
    let remote_tip_height = BlockHeight::from(5);

    let range = blockchain
        .get_unknown_block_heights((remote_tip_hash, remote_tip_height))
        .await
        .unwrap();

    assert!(
        range.is_none(),
        "Should return None when local is ahead of remote"
    );
}

#[tokio::test]
async fn test_has_canon_block_found() {
    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );
    let block_hash = genesis_block.get_hash();
    let block_hash_clone1 = block_hash.clone();

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block()
        .returning(move |_, _| Ok(Some(genesis_block.clone())));
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(block_hash.clone())));
    mock_repo
        .expect_get_height()
        .returning(|_, _| Ok(Some(BlockHeight::genesis())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let has_block = blockchain
        .has_canon_block(&block_hash_clone1)
        .await
        .unwrap();
    assert!(has_block, "Should find canon block");
}

#[tokio::test]
async fn test_has_canon_block_not_found() {
    let fake_hash = Hash::new([99u8; 32]);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo.expect_get_block().returning(|_, _| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let has_block = blockchain.has_canon_block(&fake_hash).await.unwrap();
    assert!(!has_block, "Should not find non-existent block");
}

#[tokio::test]
async fn test_has_known_block_found() {
    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block()
        .returning(move |_, _| Ok(Some(genesis_block.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let test_hash = Hash::new([1u8; 32]);
    let has_block = blockchain.has_known_block(&test_hash).await.unwrap();
    assert!(has_block, "Should find known block");
}

#[tokio::test]
async fn test_has_known_block_not_found() {
    let fake_hash = Hash::new([99u8; 32]);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo.expect_get_block().returning(|_, _| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let has_block = blockchain.has_known_block(&fake_hash).await.unwrap();
    assert!(!has_block, "Should not find non-existent block");
}

#[tokio::test]
async fn test_set_tip() {
    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );
    let block_hash = genesis_block.get_hash();
    let block_height = genesis_block.get_height();

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo.expect_set_tip().times(1).returning(|_, _| Ok(()));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    blockchain.set_tip(&genesis_block).await.unwrap();

    // Verify cache was updated by calling get_tip_info
    let tip = blockchain.get_tip_info().await.unwrap();
    assert_eq!(tip, Some((block_hash, block_height)));
}

#[tokio::test]
async fn test_get_canon_block_found() {
    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );
    let block_hash = genesis_block.get_hash();
    let block_hash_clone1 = block_hash.clone();

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block()
        .returning(move |_, _| Ok(Some(genesis_block.clone())));
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(block_hash.clone())));
    mock_repo
        .expect_get_height()
        .returning(|_, _| Ok(Some(BlockHeight::genesis())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_canon_block(&block_hash_clone1)
        .await
        .unwrap();
    assert!(result.is_some(), "Should find canonical block");
}

#[tokio::test]
async fn test_get_canon_block_not_found() {
    let fake_hash = Hash::new([99u8; 32]);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo.expect_get_block().returning(|_, _| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain.get_canon_block(&fake_hash).await.unwrap();
    assert!(result.is_none(), "Should not find non-existent block");
}

#[tokio::test]
async fn test_get_known_block() {
    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );
    let block_hash = genesis_block.get_hash();

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block()
        .returning(move |_, _| Ok(Some(genesis_block.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain.get_known_block(&block_hash).await.unwrap();
    assert!(result.is_some(), "Should find known block");
}

#[tokio::test]
async fn test_get_canon_block_by_height_found() {
    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );
    let block_hash = genesis_block.get_hash();
    let block_height = BlockHeight::genesis();
    let block_hash_clone1 = block_hash.clone();
    let block_height_clone1 = block_height.clone();
    let block_height_clone2 = block_height.clone();

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block_hash_by_height()
        .returning(move |_, _| Ok(Some(block_hash.clone())));
    mock_repo
        .expect_get_block()
        .returning(move |_, _| Ok(Some(genesis_block.clone())));
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(block_hash_clone1.clone())));
    mock_repo
        .expect_get_height()
        .returning(move |_, _| Ok(Some(block_height_clone1.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_canon_block_by_height(&block_height_clone2)
        .await
        .unwrap();
    assert!(result.is_some(), "Should find canonical block by height");
}

#[tokio::test]
async fn test_get_canon_block_by_height_not_found() {
    let block_height = BlockHeight::from(999);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block_hash_by_height()
        .returning(|_, _| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_canon_block_by_height(&block_height)
        .await
        .unwrap();
    assert!(result.is_none(), "Should not find non-existent block");
}

#[tokio::test]
async fn test_get_known_block_by_height_found() {
    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );
    let block_hash = genesis_block.get_hash();
    let block_height = BlockHeight::genesis();

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block_hash_by_height()
        .returning(move |_, _| Ok(Some(block_hash.clone())));
    mock_repo
        .expect_get_block()
        .returning(move |_, _| Ok(Some(genesis_block.clone())));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_known_block_by_height(&block_height)
        .await
        .unwrap();
    assert!(result.is_some(), "Should find known block by height");
}

#[tokio::test]
async fn test_get_known_block_by_height_not_found() {
    let block_height = BlockHeight::from(999);

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block_hash_by_height()
        .returning(|_, _| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_known_block_by_height(&block_height)
        .await
        .unwrap();
    assert!(result.is_none(), "Should not find non-existent block");
}

#[tokio::test]
async fn test_get_known_blocks_by_height_range() {
    let block1_hash = Hash::new([1u8; 32]);
    let block2_hash = Hash::new([2u8; 32]);
    let block3_hash = Hash::new([3u8; 32]);

    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_block_hashes_by_height_range()
        .returning(move |_| {
            Ok(vec![
                block1_hash.clone(),
                block2_hash.clone(),
                block3_hash.clone(),
            ])
        });
    mock_repo.expect_get_multiple_blocks().returning(move |_| {
        Ok(vec![
            genesis_block.clone(),
            genesis_block.clone(),
            genesis_block.clone(),
        ])
    });

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_known_blocks_by_height_range(BlockHeight::from(0)..=BlockHeight::from(2))
        .await
        .unwrap();

    assert_eq!(result.len(), 3, "Should return 3 blocks");
}

#[tokio::test]
async fn test_get_known_blocks_by_height_range_count_mismatch() {
    let block1_hash = Hash::new([1u8; 32]);

    let mut mock_repo = MockBlockchainRepository::new();
    // Return only 1 hash when 3 are expected
    mock_repo
        .expect_get_block_hashes_by_height_range()
        .returning(move |_| Ok(vec![block1_hash.clone()]));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_known_blocks_by_height_range(BlockHeight::from(0)..=BlockHeight::from(2))
        .await;

    assert!(result.is_err(), "Should return error on count mismatch");
}

#[tokio::test]
async fn test_get_canon_blocks_by_height_range_within_tip() {
    let block1_hash = Hash::new([1u8; 32]);
    let block2_hash = Hash::new([2u8; 32]);
    let tip_hash = Hash::new([10u8; 32]);

    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(tip_hash.clone())));
    mock_repo
        .expect_get_height()
        .returning(|_, _| Ok(Some(BlockHeight::from(10))));
    mock_repo
        .expect_get_block_hashes_by_height_range()
        .returning(move |_| Ok(vec![block1_hash.clone(), block2_hash.clone()]));
    mock_repo
        .expect_get_multiple_blocks()
        .returning(move |_| Ok(vec![genesis_block.clone(), genesis_block.clone()]));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    let result = blockchain
        .get_canon_blocks_by_height_range(BlockHeight::from(0)..=BlockHeight::from(1))
        .await
        .unwrap();

    assert_eq!(result.len(), 2, "Should return 2 blocks within tip range");
}

#[tokio::test]
async fn test_get_canon_blocks_by_height_range_beyond_tip() {
    let tip_hash = Hash::new([5u8; 32]);

    let genesis_block = Block::_new_validated(
        crate::entities::block::NonValidatedBlock::new_genesis(create_test_genesis_config())
            .unwrap(),
    );

    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo
        .expect_get_tip()
        .returning(move |_| Ok(Some(tip_hash.clone())));
    mock_repo
        .expect_get_height()
        .returning(|_, _| Ok(Some(BlockHeight::from(5))));

    // Mock expects the capped range (0-5 = 6 blocks)
    mock_repo
        .expect_get_block_hashes_by_height_range()
        .returning(move |_| {
            // Return 6 block hashes for the capped range
            Ok((0..=5).map(|i| Hash::new([i as u8; 32])).collect())
        });

    mock_repo
        .expect_get_multiple_blocks()
        .returning(move |hashes| {
            // Return the same number of blocks as hashes
            Ok(vec![genesis_block.clone(); hashes.len()])
        });

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    // Request 0-10 (11 blocks expected), but tip is at 5 (only 6 blocks available)
    // This should throw an error because expected_len is 11, but we only fetched 6 blocks
    let result = blockchain
        .get_canon_blocks_by_height_range(BlockHeight::from(0)..=BlockHeight::from(10))
        .await;

    assert!(
        result.is_err(),
        "Should return error when requested range exceeds tip"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Block count mismatch"),
        "Error should mention block count mismatch"
    );
}

#[tokio::test]
async fn test_get_canon_blocks_by_height_range_empty_blockchain() {
    let mut mock_repo = MockBlockchainRepository::new();
    mock_repo.expect_get_tip().returning(|_| Ok(None));

    let mock_outbox = MockOutboxRepository::new();
    let blockchain = DefaultBlockchain::new(Arc::new(mock_repo), Arc::new(mock_outbox));

    // When blockchain is empty, it returns empty vec from the async block,
    // but expected_len is calculated from the input range (0-5 = 6 blocks)
    // This causes a mismatch: 0 blocks returned vs 6 expected -> throws error
    let result = blockchain
        .get_canon_blocks_by_height_range(BlockHeight::from(0)..=BlockHeight::from(5))
        .await;

    assert!(
        result.is_err(),
        "Should return error when blockchain is empty but blocks are requested"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Block count mismatch"),
        "Error should mention block count mismatch"
    );
}
