use super::*;
use crate::entities::block::{Block, BlockDifficultyTarget, BlockTemplate, NonValidatedBlock};
use crate::entities::transaction::{
    NonValidatedTransaction, Transaction, TransactionAmount, TransactionInput, TransactionOutPoint,
    TransactionOutput,
};
use crate::genesis::config::{GenesisConfig, GenesisConfigUtxoFunds};
use crate::repos::utxo::MockUtxoRepository;
use crate::types::hash::Hash;
use crate::types::sign::PublicKey;
use crate::types::time::DateTime;
use crate::types::wallet::WalletAddress;
use common::error::AppError;
use common::tx::ctx::AtomicTransactionContext;
use common::tx::{AtomicTransactionOutput, UnitOfWork};
use std::any::TypeId;
use std::cell::RefCell;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

fn create_test_genesis_config() -> GenesisConfig {
    let wallet_pub_key = "59f783b83cf3b6552f53044743ac3454a84ed9b47897ef1576e64662363dbd6b"
        .parse::<PublicKey>()
        .expect("Valid public key");

    let utxo =
        GenesisConfigUtxoFunds::new_unchecked(wallet_pub_key, TransactionAmount::new(1000000000));

    let timestamp = DateTime::from_ms(1725799696000);

    GenesisConfig::new_unchecked(vec![utxo], timestamp)
}

fn create_test_block_with_transactions(txs: Vec<Transaction>) -> Block {
    let prev_block = Block::_new_validated(
        NonValidatedBlock::new_genesis(create_test_genesis_config()).unwrap(),
    );

    let non_validated_txs: Vec<NonValidatedTransaction> =
        txs.into_iter().map(|tx| tx.invalidate()).collect();

    let template = BlockTemplate::new(
        &prev_block,
        non_validated_txs,
        BlockDifficultyTarget::_new_stub(),
    );

    let non_validated_block = NonValidatedBlock::from_template(template).unwrap();
    Block::_new_validated(non_validated_block)
}

fn create_mock_transaction_with_inputs_outputs(
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    seed: u8,
) -> Transaction {
    let timestamp = DateTime::from_ms(1000000000 + (seed as u64) * 1000);
    let tx = NonValidatedTransaction::new(inputs, outputs, timestamp).unwrap();
    Transaction::_new_validated(tx)
}

fn create_test_output(amount: u128) -> TransactionOutput {
    let wallet_address =
        WalletAddress::from_str("54b73c091395a30874a397cbfcd54c7348175a01ee6ccf0a1133f8f8b3a19e7d")
            .unwrap();
    TransactionOutput::new(wallet_address, TransactionAmount::new(amount))
}

fn create_test_input(prev_outpoint: TransactionOutPoint) -> TransactionInput {
    TransactionInput::new(prev_outpoint)
}

// Mockall doesn't support mocking Fn objects. See https://github.com/asomers/mockall/issues/139.
#[derive(Debug)]
struct MockUnitOfWork;

impl UnitOfWork for MockUnitOfWork {
    fn run_in_transaction(
        &self,
        f: Box<
            dyn for<'a> FnMut(
                    &'a mut dyn AtomicTransactionContext,
                ) -> Result<AtomicTransactionOutput, AppError>
                + Send,
        >,
    ) -> Result<AtomicTransactionOutput, AppError> {
        struct MockTxCtx;
        impl AtomicTransactionContext for MockTxCtx {
            fn type_id(&self) -> TypeId {
                TypeId::of::<MockTxCtx>()
            }

            fn as_any(&self) -> Box<dyn std::any::Any> {
                let ctx = MockTxCtx;
                Box::new(ctx)
            }
        }
        let f = RefCell::new(f);
        let mut ctx = MockTxCtx;
        f.borrow_mut()(&mut ctx)
    }
}

#[test]
fn test_apply_block_with_outputs_only() {
    let deleted = Arc::new(Mutex::new(Vec::new()));
    let inserted = Arc::new(Mutex::new(Vec::new()));

    let deleted_clone = deleted.clone();
    let inserted_clone = inserted.clone();

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo
        .expect_delete_utxo()
        .returning(move |_, outpoint| {
            deleted_clone.lock().unwrap().push(outpoint.clone());
            Ok(())
        });

    mock_repo.expect_insert_utxo().returning(move |_, utxo| {
        inserted_clone.lock().unwrap().push(utxo.clone());
        Ok(())
    });

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    // Create a transaction with outputs but no inputs
    let outputs = vec![create_test_output(100), create_test_output(200)];
    let tx = create_mock_transaction_with_inputs_outputs(vec![], outputs, 1);
    let block = create_test_block_with_transactions(vec![tx.clone()]);

    let result = service.apply_block(&block);

    assert!(result.is_ok(), "Should apply block successfully");
    assert_eq!(
        deleted.lock().unwrap().len(),
        0,
        "Should not delete any UTXOs"
    );
    assert_eq!(inserted.lock().unwrap().len(), 2, "Should insert 2 UTXOs");
}

#[test]
fn test_apply_block_with_inputs_and_outputs() {
    let deleted = Arc::new(Mutex::new(Vec::new()));
    let inserted = Arc::new(Mutex::new(Vec::new()));

    let deleted_clone = deleted.clone();
    let inserted_clone = inserted.clone();

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo
        .expect_delete_utxo()
        .returning(move |_, outpoint| {
            deleted_clone.lock().unwrap().push(outpoint.clone());
            Ok(())
        });

    mock_repo.expect_insert_utxo().returning(move |_, utxo| {
        inserted_clone.lock().unwrap().push(utxo.clone());
        Ok(())
    });

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    // Create a transaction that spends 2 UTXOs and creates 3 new ones
    let prev_tx_hash = Hash::new([1u8; 32]);
    let inputs = vec![
        create_test_input(TransactionOutPoint::new(prev_tx_hash.clone(), 0)),
        create_test_input(TransactionOutPoint::new(prev_tx_hash, 1)),
    ];
    let outputs = vec![
        create_test_output(50),
        create_test_output(150),
        create_test_output(90),
    ];
    let tx = create_mock_transaction_with_inputs_outputs(inputs, outputs, 1);
    let block = create_test_block_with_transactions(vec![tx]);

    let result = service.apply_block(&block);

    assert!(result.is_ok(), "Should apply block successfully");
    assert_eq!(deleted.lock().unwrap().len(), 2, "Should delete 2 UTXOs");
    assert_eq!(inserted.lock().unwrap().len(), 3, "Should insert 3 UTXOs");
}

#[test]
fn test_apply_block_with_multiple_transactions() {
    let deleted = Arc::new(Mutex::new(Vec::new()));
    let inserted = Arc::new(Mutex::new(Vec::new()));

    let deleted_clone = deleted.clone();
    let inserted_clone = inserted.clone();

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo
        .expect_delete_utxo()
        .returning(move |_, outpoint| {
            deleted_clone.lock().unwrap().push(outpoint.clone());
            Ok(())
        });

    mock_repo.expect_insert_utxo().returning(move |_, utxo| {
        inserted_clone.lock().unwrap().push(utxo.clone());
        Ok(())
    });

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    // Transaction 1: 1 input, 2 outputs
    let prev_tx_hash1 = Hash::new([1u8; 32]);
    let inputs1 = vec![create_test_input(TransactionOutPoint::new(
        prev_tx_hash1.clone(),
        0,
    ))];
    let tx1 = create_mock_transaction_with_inputs_outputs(
        inputs1,
        vec![create_test_output(40), create_test_output(50)],
        1,
    );

    // Transaction 2: 2 inputs, 1 output
    let prev_tx_hash2a = Hash::new([2u8; 32]);
    let prev_tx_hash2b = Hash::new([2u8; 32]);
    let inputs2 = vec![
        create_test_input(TransactionOutPoint::new(prev_tx_hash2a.clone(), 0)),
        create_test_input(TransactionOutPoint::new(prev_tx_hash2b.clone(), 0)),
    ];
    let tx2 =
        create_mock_transaction_with_inputs_outputs(inputs2, vec![create_test_output(400)], 2);

    let block = create_test_block_with_transactions(vec![tx1, tx2]);

    let result = service.apply_block(&block);

    assert!(result.is_ok(), "Should apply block successfully");
    assert_eq!(
        deleted.lock().unwrap().len(),
        3,
        "Should delete 3 UTXOs (1 + 2)"
    );
    assert_eq!(
        inserted.lock().unwrap().len(),
        3,
        "Should insert 3 UTXOs (2 + 1)"
    );
}

#[test]
fn test_apply_block_delete_error() {
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo
        .expect_delete_utxo()
        .returning(|_, _| Err(AppError::internal("Delete failed")));

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    let prev_tx_hash = Hash::new([1u8; 32]);
    let inputs = vec![create_test_input(TransactionOutPoint::new(
        prev_tx_hash.clone(),
        0,
    ))];
    let tx = create_mock_transaction_with_inputs_outputs(inputs, vec![], 1);
    let block = create_test_block_with_transactions(vec![tx]);

    let result = service.apply_block(&block);

    assert!(result.is_err(), "Should propagate delete error");
}

#[test]
fn test_apply_block_insert_error() {
    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo.expect_delete_utxo().returning(|_, _| Ok(()));

    mock_repo
        .expect_insert_utxo()
        .returning(|_, _| Err(AppError::internal("Insert failed")));

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    let outputs = vec![create_test_output(100)];
    let tx = create_mock_transaction_with_inputs_outputs(vec![], outputs, 1);
    let block = create_test_block_with_transactions(vec![tx]);

    let result = service.apply_block(&block);

    assert!(result.is_err(), "Should propagate insert error");
}

#[test]
fn test_apply_block_correct_outpoint_indices() {
    let inserted = Arc::new(Mutex::new(Vec::new()));
    let inserted_clone = inserted.clone();

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo.expect_insert_utxo().returning(move |_, utxo| {
        inserted_clone.lock().unwrap().push(utxo.clone());
        Ok(())
    });

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    // Create transaction with 3 outputs
    let outputs = vec![
        create_test_output(100),
        create_test_output(200),
        create_test_output(300),
    ];
    let tx = create_mock_transaction_with_inputs_outputs(vec![], outputs, 1);
    let tx_hash = tx.get_hash();
    let block = create_test_block_with_transactions(vec![tx]);

    service.apply_block(&block).unwrap();

    let inserted_utxos = inserted.lock().unwrap();
    assert_eq!(inserted_utxos.len(), 3, "Should insert 3 UTXOs");

    // Verify indices are correct
    assert_eq!(inserted_utxos[0].get_outpoint().get_tx_id(), &tx_hash);
    assert_eq!(inserted_utxos[0].get_outpoint().get_tx_output_index(), 0);

    assert_eq!(inserted_utxos[1].get_outpoint().get_tx_id(), &tx_hash);
    assert_eq!(inserted_utxos[1].get_outpoint().get_tx_output_index(), 1);

    assert_eq!(inserted_utxos[2].get_outpoint().get_tx_id(), &tx_hash);
    assert_eq!(inserted_utxos[2].get_outpoint().get_tx_output_index(), 2);
}

#[test]
fn test_apply_block_preserves_output_amounts() {
    let inserted = Arc::new(Mutex::new(Vec::new()));
    let inserted_clone = inserted.clone();

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo.expect_insert_utxo().returning(move |_, utxo| {
        inserted_clone.lock().unwrap().push(utxo.clone());
        Ok(())
    });

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    let outputs = vec![
        create_test_output(123),
        create_test_output(456),
        create_test_output(789),
    ];
    let tx = create_mock_transaction_with_inputs_outputs(vec![], outputs, 1);
    let block = create_test_block_with_transactions(vec![tx]);

    service.apply_block(&block).unwrap();

    let inserted_utxos = inserted.lock().unwrap();
    assert_eq!(inserted_utxos[0].get_output().get_amount().as_u128(), 123);
    assert_eq!(inserted_utxos[1].get_output().get_amount().as_u128(), 456);
    assert_eq!(inserted_utxos[2].get_output().get_amount().as_u128(), 789);
}

#[test]
fn test_apply_block_deletes_correct_outpoints() {
    let deleted = Arc::new(Mutex::new(Vec::new()));
    let deleted_clone = deleted.clone();

    let mut mock_repo = MockUtxoRepository::new();
    mock_repo
        .expect_get_utxo_set_append_block_unit_of_work()
        .returning(|| Arc::new(MockUnitOfWork));

    mock_repo
        .expect_delete_utxo()
        .returning(move |_, outpoint| {
            deleted_clone.lock().unwrap().push(outpoint.clone());
            Ok(())
        });

    let service = UtxoSetWriterService::new(Arc::new(mock_repo));

    let prev_tx_hash1 = Hash::new([1u8; 32]);
    let prev_tx_hash2 = Hash::new([2u8; 32]);
    let inputs = vec![
        create_test_input(TransactionOutPoint::new(prev_tx_hash1.clone(), 5)),
        create_test_input(TransactionOutPoint::new(prev_tx_hash2.clone(), 3)),
    ];
    let tx = create_mock_transaction_with_inputs_outputs(inputs, vec![], 1);
    let block = create_test_block_with_transactions(vec![tx]);

    service.apply_block(&block).unwrap();

    let deleted_outpoints = deleted.lock().unwrap();
    assert_eq!(deleted_outpoints.len(), 2, "Should delete 2 outpoints");

    assert_eq!(deleted_outpoints[0].get_tx_id(), &prev_tx_hash1);
    assert_eq!(deleted_outpoints[0].get_tx_output_index(), 5);

    assert_eq!(deleted_outpoints[1].get_tx_id(), &prev_tx_hash2);
    assert_eq!(deleted_outpoints[1].get_tx_output_index(), 3);
}
