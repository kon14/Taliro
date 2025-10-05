use super::*;
use crate::entities::transaction::{
    TransactionAmount, TransactionOutPoint, TransactionOutput, Utxo,
};
use crate::repos::utxo::MockUtxoRepository;
use crate::types::hash::Hash;
use crate::types::wallet::WalletAddress;
use common::error::AppError;
use std::str::FromStr;
use std::sync::Arc;

fn create_test_outpoint(seed: u8) -> TransactionOutPoint {
    let tx_hash = Hash::new([seed; 32]);
    TransactionOutPoint::new(tx_hash, seed as usize)
}

fn create_test_output(amount: u128) -> TransactionOutput {
    let wallet_address =
        WalletAddress::from_str("54b73c091395a30874a397cbfcd54c7348175a01ee6ccf0a1133f8f8b3a19e7d")
            .unwrap();
    TransactionOutput::new(wallet_address, TransactionAmount::new(amount))
}

fn create_test_utxo(seed: u8, amount: u128) -> Utxo {
    let outpoint = create_test_outpoint(seed);
    let output = create_test_output(amount);
    Utxo::new(outpoint, output)
}

#[test]
fn test_get_utxo_found() {
    let outpoint = create_test_outpoint(1);
    let output = create_test_output(100);
    let outpoint_clone = outpoint.clone();

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_output()
        .returning(move |_, _| Ok(Some(output.clone())));

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let result = service.get_utxo(&outpoint_clone).unwrap();

    assert!(result.is_some(), "Should find UTXO");
    let utxo = result.unwrap();
    assert_eq!(utxo.get_outpoint(), &outpoint_clone);
    assert_eq!(utxo.get_output().get_amount().as_u128(), 100);
}

#[test]
fn test_get_utxo_not_found() {
    let outpoint = create_test_outpoint(1);

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo.expect_get_output().returning(|_, _| Ok(None));

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let result = service.get_utxo(&outpoint).unwrap();

    assert!(result.is_none(), "Should return None for non-existent UTXO");
}

#[test]
fn test_get_utxo_repository_error() {
    let outpoint = create_test_outpoint(1);

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_output()
        .returning(|_, _| Err(AppError::internal("Database error")));

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let result = service.get_utxo(&outpoint);

    assert!(result.is_err(), "Should propagate repository error");
}

#[test]
fn test_get_multiple_utxos_by_outpoints_all_found() {
    let outpoint1 = create_test_outpoint(1);
    let outpoint2 = create_test_outpoint(2);
    let outpoint3 = create_test_outpoint(3);

    let output1 = create_test_output(100);
    let output2 = create_test_output(200);
    let output3 = create_test_output(300);

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo.expect_get_output().returning(move |_, op| {
        if op.get_tx_output_index() == 1 {
            Ok(Some(output1.clone()))
        } else if op.get_tx_output_index() == 2 {
            Ok(Some(output2.clone()))
        } else if op.get_tx_output_index() == 3 {
            Ok(Some(output3.clone()))
        } else {
            Ok(None)
        }
    });

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let outpoints = vec![outpoint1.clone(), outpoint2.clone(), outpoint3.clone()];
    let result = service.get_multiple_utxos_by_outpoints(&outpoints).unwrap();

    assert_eq!(result.len(), 3, "Should return all 3 UTXOs");
    assert_eq!(result[0].get_output().get_amount().as_u128(), 100);
    assert_eq!(result[1].get_output().get_amount().as_u128(), 200);
    assert_eq!(result[2].get_output().get_amount().as_u128(), 300);
}

#[test]
fn test_get_multiple_utxos_by_outpoints_some_not_found() {
    let outpoint1 = create_test_outpoint(1);
    let outpoint2 = create_test_outpoint(2);
    let outpoint3 = create_test_outpoint(3);

    let output1 = create_test_output(100);
    let output3 = create_test_output(300);

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo.expect_get_output().returning(move |_, op| {
        if op.get_tx_output_index() == 1 {
            Ok(Some(output1.clone()))
        } else if op.get_tx_output_index() == 3 {
            Ok(Some(output3.clone()))
        } else {
            Ok(None) // outpoint2 not found
        }
    });

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let outpoints = vec![outpoint1, outpoint2, outpoint3];
    let result = service.get_multiple_utxos_by_outpoints(&outpoints).unwrap();

    assert_eq!(result.len(), 2, "Should return only 2 found UTXOs");
    assert_eq!(result[0].get_output().get_amount().as_u128(), 100);
    assert_eq!(result[1].get_output().get_amount().as_u128(), 300);
}

#[test]
fn test_get_multiple_utxos_by_outpoints_empty_input() {
    let mock_repo = MockUtxoRepository::new();
    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let outpoints = vec![];
    let result = service.get_multiple_utxos_by_outpoints(&outpoints).unwrap();

    assert_eq!(result.len(), 0, "Should return empty vec for empty input");
}

#[test]
fn test_get_multiple_utxos_by_outpoints_error_propagation() {
    let outpoint1 = create_test_outpoint(1);
    let outpoint2 = create_test_outpoint(2);

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo.expect_get_output().returning(|_, op| {
        if op.get_tx_output_index() == 1 {
            Ok(Some(create_test_output(100)))
        } else {
            Err(AppError::internal("Database error"))
        }
    });

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let outpoints = vec![outpoint1, outpoint2];
    let result = service.get_multiple_utxos_by_outpoints(&outpoints);

    assert!(result.is_err(), "Should propagate error from repository");
}

#[test]
fn test_get_multiple_utxos() {
    let utxo1 = create_test_utxo(1, 100);
    let utxo2 = create_test_utxo(2, 200);
    let utxo3 = create_test_utxo(3, 300);

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_multiple_utxos()
        .returning(move || Ok(vec![utxo1.clone(), utxo2.clone(), utxo3.clone()]));

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let result = service.get_multiple_utxos().unwrap();

    assert_eq!(result.len(), 3, "Should return all UTXOs");
    assert_eq!(result[0].get_output().get_amount().as_u128(), 100);
    assert_eq!(result[1].get_output().get_amount().as_u128(), 200);
    assert_eq!(result[2].get_output().get_amount().as_u128(), 300);
}

#[test]
fn test_get_multiple_utxos_empty() {
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_multiple_utxos()
        .returning(|| Ok(vec![]));

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let result = service.get_multiple_utxos().unwrap();

    assert_eq!(
        result.len(),
        0,
        "Should return empty vec when no UTXOs exist"
    );
}

#[test]
fn test_get_multiple_utxos_error() {
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_multiple_utxos()
        .returning(|| Err(AppError::internal("Database error")));

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let result = service.get_multiple_utxos();

    assert!(result.is_err(), "Should propagate error from repository");
}

#[test]
fn test_get_utxo_count_zero() {
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo.expect_get_utxo_count().returning(|| 0);

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let count = service.get_utxo_count();

    assert_eq!(count, 0, "Should return 0 for empty UTXO set");
}

#[test]
fn test_get_utxo_count_non_zero() {
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo.expect_get_utxo_count().returning(|| 42);

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    let count = service.get_utxo_count();

    assert_eq!(count, 42, "Should return correct UTXO count");
}

#[test]
fn test_get_utxo_with_different_amounts() {
    let outpoint = create_test_outpoint(1);

    // Test with zero amount
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_output()
        .returning(|_, _| Ok(Some(create_test_output(0))));

    let service = UtxoReaderService::new(Arc::new(mock_repo));
    let result = service.get_utxo(&outpoint).unwrap();
    assert_eq!(result.unwrap().get_output().get_amount().as_u128(), 0);

    // Test with large amount
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_output()
        .returning(|_, _| Ok(Some(create_test_output(u128::MAX))));

    let service = UtxoReaderService::new(Arc::new(mock_repo));
    let result = service.get_utxo(&outpoint).unwrap();
    assert_eq!(
        result.unwrap().get_output().get_amount().as_u128(),
        u128::MAX
    );
}

#[test]
fn test_multiple_calls_to_get_utxo() {
    let outpoint1 = create_test_outpoint(1);
    let outpoint2 = create_test_outpoint(2);

    let output1 = create_test_output(100);
    let output2 = create_test_output(200);

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo.expect_get_output().returning(move |_, op| {
        if op.get_tx_output_index() == 1 {
            Ok(Some(output1.clone()))
        } else if op.get_tx_output_index() == 2 {
            Ok(Some(output2.clone()))
        } else {
            Ok(None)
        }
    });

    let service = UtxoReaderService::new(Arc::new(mock_repo));

    // First call
    let result1 = service.get_utxo(&outpoint1).unwrap();
    assert!(result1.is_some());
    assert_eq!(result1.unwrap().get_output().get_amount().as_u128(), 100);

    // Second call with different outpoint
    let result2 = service.get_utxo(&outpoint2).unwrap();
    assert!(result2.is_some());
    assert_eq!(result2.unwrap().get_output().get_amount().as_u128(), 200);
}
