use super::*;
use crate::entities::transaction::{
    NonValidatedTransaction, TransactionAmount, TransactionInput, TransactionOutPoint,
    TransactionOutput, Utxo,
};
use crate::system::utxo::MockUtxoSetReader;
use crate::types::hash::Hash;
use crate::types::time::DateTime;
use crate::types::wallet::WalletAddress;
use common::error::TransactionValidationError;
use std::str::FromStr;

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

fn create_test_utxo(seed: u8, amount: u128) -> Utxo {
    let outpoint = create_test_outpoint(seed, 0);
    let output = TransactionOutput::new(
        create_test_wallet_address(seed),
        TransactionAmount::new(amount),
    );
    Utxo::new(outpoint, output)
}

fn create_mock_validator() -> DefaultTransactionValidator {
    let mock_utxo_reader = MockUtxoSetReader::new();
    DefaultTransactionValidator::new(Arc::new(mock_utxo_reader))
}

// ============================================================================
// validate_structure()
// ============================================================================

#[test]
fn test_validate_structure_empty_inputs() {
    let validator = create_mock_validator();

    let output = TransactionOutput::new(create_test_wallet_address(1), TransactionAmount::new(100));
    let tx =
        NonValidatedTransaction::new(vec![], vec![output], DateTime::from_ms(1000000000)).unwrap();

    let result = validator.pub_validate_structure(&tx);
    assert!(result.is_err(), "Should fail with empty inputs");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::EmptyInputs { .. }) => {}
        _ => panic!("Expected EmptyInputs error"),
    }
}

#[test]
fn test_validate_structure_empty_outputs() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let tx =
        NonValidatedTransaction::new(vec![input], vec![], DateTime::from_ms(1000000000)).unwrap();

    let result = validator.pub_validate_structure(&tx);
    assert!(result.is_err(), "Should fail with empty outputs");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::EmptyOutputs { .. }) => {}
        _ => panic!("Expected EmptyOutputs error"),
    }
}

#[test]
fn test_validate_structure_valid() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output = TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(100));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    let result = validator.pub_validate_structure(&tx);
    assert!(result.is_ok(), "Valid structure should pass");
}

#[test]
fn test_validate_structure_multiple_inputs_and_outputs() {
    let validator = create_mock_validator();

    let input1 = TransactionInput::new(create_test_outpoint(1, 0));
    let input2 = TransactionInput::new(create_test_outpoint(2, 0));
    let output1 =
        TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(100));
    let output2 =
        TransactionOutput::new(create_test_wallet_address(4), TransactionAmount::new(200));
    let tx = NonValidatedTransaction::new(
        vec![input1, input2],
        vec![output1, output2],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    let result = validator.pub_validate_structure(&tx);
    assert!(
        result.is_ok(),
        "Valid structure with multiple inputs/outputs should pass"
    );
}

// ============================================================================
// validate_output_values()
// ============================================================================

#[test]
fn test_validate_output_values_zero_amount() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output = TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(0));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    let result = validator.pub_validate_output_values(&tx);
    assert!(result.is_err(), "Should fail with zero output amount");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::InvalidOutputAmount {
            index,
            ..
        }) => {
            assert_eq!(index, 0, "Should report first output as invalid");
        }
        _ => panic!("Expected InvalidOutputAmount error"),
    }
}

#[test]
fn test_validate_output_values_all_valid() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output1 =
        TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(100));
    let output2 =
        TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(200));
    let output3 =
        TransactionOutput::new(create_test_wallet_address(4), TransactionAmount::new(300));
    let tx = NonValidatedTransaction::new(
        vec![input],
        vec![output1, output2, output3],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    let result = validator.pub_validate_output_values(&tx);
    assert!(result.is_ok(), "All valid output amounts should pass");
}

#[test]
fn test_validate_output_values_one_zero_among_multiple() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output1 =
        TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(100));
    let output2 = TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(0));
    let output3 =
        TransactionOutput::new(create_test_wallet_address(4), TransactionAmount::new(300));
    let tx = NonValidatedTransaction::new(
        vec![input],
        vec![output1, output2, output3],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    let result = validator.pub_validate_output_values(&tx);
    assert!(result.is_err(), "Should fail with one zero output");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::InvalidOutputAmount {
            index,
            ..
        }) => {
            assert_eq!(index, 1, "Should report second output as invalid");
        }
        _ => panic!("Expected InvalidOutputAmount error"),
    }
}

#[test]
fn test_validate_output_values_single_valid_output() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output = TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(1));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    let result = validator.pub_validate_output_values(&tx);
    assert!(
        result.is_ok(),
        "Minimal valid output (amount=1) should pass"
    );
}

// ============================================================================
// validate_balance()
// ============================================================================

#[test]
fn test_validate_balance_outputs_exceed_inputs() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output =
        TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(1500));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    // Provide input UTXOs with less value than outputs
    let input_utxos = vec![create_test_utxo(1, 1000)];

    let result = validator.pub_validate_balance(&tx, &input_utxos);
    assert!(result.is_err(), "Should fail when outputs exceed inputs");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::OutputsExceedInputs {
            inputs,
            outputs,
            ..
        }) => {
            assert_eq!(inputs, 1000, "Input sum should be 1000");
            assert_eq!(outputs, 1500, "Output sum should be 1500");
        }
        _ => panic!("Expected OutputsExceedInputs error"),
    }
}

#[test]
fn test_validate_balance_exact_match() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output =
        TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(1000));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    let input_utxos = vec![create_test_utxo(1, 1000)];

    let result = validator.pub_validate_balance(&tx, &input_utxos);
    assert!(result.is_ok(), "Exact balance (no fee) should pass");
}

#[test]
fn test_validate_balance_with_fee() {
    let validator = create_mock_validator();

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output = TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(900));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    let input_utxos = vec![create_test_utxo(1, 1000)];

    let result = validator.pub_validate_balance(&tx, &input_utxos);
    assert!(
        result.is_ok(),
        "Inputs greater than outputs (with fee) should pass"
    );
}

#[test]
fn test_validate_balance_multiple_inputs_and_outputs() {
    let validator = create_mock_validator();

    let input1 = TransactionInput::new(create_test_outpoint(1, 0));
    let input2 = TransactionInput::new(create_test_outpoint(2, 0));
    let output1 =
        TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(400));
    let output2 =
        TransactionOutput::new(create_test_wallet_address(4), TransactionAmount::new(500));
    let tx = NonValidatedTransaction::new(
        vec![input1, input2],
        vec![output1, output2],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    // Total inputs: 600 + 500 = 1100, Total outputs: 400 + 500 = 900 (fee: 200)
    let input_utxos = vec![create_test_utxo(1, 600), create_test_utxo(2, 500)];

    let result = validator.pub_validate_balance(&tx, &input_utxos);
    assert!(
        result.is_ok(),
        "Multiple inputs/outputs with valid balance should pass"
    );
}

#[test]
fn test_validate_balance_multiple_inputs_insufficient() {
    let validator = create_mock_validator();

    let input1 = TransactionInput::new(create_test_outpoint(1, 0));
    let input2 = TransactionInput::new(create_test_outpoint(2, 0));
    let output =
        TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(1000));
    let tx = NonValidatedTransaction::new(
        vec![input1, input2],
        vec![output],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    // Total inputs: 300 + 400 = 700, but outputs need 1000
    let input_utxos = vec![create_test_utxo(1, 300), create_test_utxo(2, 400)];

    let result = validator.pub_validate_balance(&tx, &input_utxos);
    assert!(result.is_err(), "Insufficient inputs should fail");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::OutputsExceedInputs {
            inputs,
            outputs,
            ..
        }) => {
            assert_eq!(inputs, 700);
            assert_eq!(outputs, 1000);
        }
        _ => panic!("Expected OutputsExceedInputs error"),
    }
}

// ============================================================================
// validate_inputs_unspent()
// ============================================================================

#[tokio::test]
async fn test_validate_inputs_unspent_single_utxo_found() {
    let mut mock_utxo_reader = MockUtxoSetReader::new();

    let outpoint = create_test_outpoint(1, 0);
    let utxo = create_test_utxo(1, 1000);

    mock_utxo_reader
        .expect_get_utxo()
        .withf(move |op| op == &outpoint)
        .times(1)
        .returning(move |_| Ok(Some(utxo.clone())));

    let validator = DefaultTransactionValidator::new(Arc::new(mock_utxo_reader));

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output = TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(100));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    let result = validator.pub_validate_inputs_unspent(&tx).await;
    assert!(result.is_ok(), "Should find the UTXO");

    let utxos = result.unwrap();
    assert_eq!(utxos.len(), 1, "Should return one UTXO");
    assert_eq!(utxos[0].get_output().get_amount().as_u128(), 1000);
}

#[tokio::test]
async fn test_validate_inputs_unspent_utxo_not_found() {
    let mut mock_utxo_reader = MockUtxoSetReader::new();

    let outpoint = create_test_outpoint(1, 0);

    mock_utxo_reader
        .expect_get_utxo()
        .withf(move |op| op == &outpoint)
        .times(1)
        .returning(|_| Ok(None));

    let validator = DefaultTransactionValidator::new(Arc::new(mock_utxo_reader));

    let input = TransactionInput::new(create_test_outpoint(1, 0));
    let output = TransactionOutput::new(create_test_wallet_address(2), TransactionAmount::new(100));
    let tx = NonValidatedTransaction::new(vec![input], vec![output], DateTime::from_ms(1000000000))
        .unwrap();

    let result = validator.pub_validate_inputs_unspent(&tx).await;
    assert!(result.is_err(), "Should fail when UTXO not found");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::InputUtxoNotFound {
            ..
        }) => {}
        _ => panic!("Expected InputUtxoNotFound error"),
    }
}

#[tokio::test]
async fn test_validate_inputs_unspent_multiple_utxos_all_found() {
    let mut mock_utxo_reader = MockUtxoSetReader::new();

    let outpoint1 = create_test_outpoint(1, 0);
    let outpoint2 = create_test_outpoint(2, 0);
    let utxo1 = create_test_utxo(1, 500);
    let utxo2 = create_test_utxo(2, 700);

    mock_utxo_reader
        .expect_get_utxo()
        .withf(move |op| op == &outpoint1)
        .times(1)
        .returning(move |_| Ok(Some(utxo1.clone())));

    mock_utxo_reader
        .expect_get_utxo()
        .withf(move |op| op == &outpoint2)
        .times(1)
        .returning(move |_| Ok(Some(utxo2.clone())));

    let validator = DefaultTransactionValidator::new(Arc::new(mock_utxo_reader));

    let input1 = TransactionInput::new(create_test_outpoint(1, 0));
    let input2 = TransactionInput::new(create_test_outpoint(2, 0));
    let output = TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(100));
    let tx = NonValidatedTransaction::new(
        vec![input1, input2],
        vec![output],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    let result = validator.pub_validate_inputs_unspent(&tx).await;
    assert!(result.is_ok(), "Should find all UTXOs");

    let utxos = result.unwrap();
    assert_eq!(utxos.len(), 2, "Should return two UTXOs");
    assert_eq!(utxos[0].get_output().get_amount().as_u128(), 500);
    assert_eq!(utxos[1].get_output().get_amount().as_u128(), 700);
}

#[tokio::test]
async fn test_validate_inputs_unspent_second_utxo_not_found() {
    let mut mock_utxo_reader = MockUtxoSetReader::new();

    let outpoint1 = create_test_outpoint(1, 0);
    let outpoint2 = create_test_outpoint(2, 0);
    let utxo1 = create_test_utxo(1, 500);

    mock_utxo_reader
        .expect_get_utxo()
        .withf(move |op| op == &outpoint1)
        .times(1)
        .returning(move |_| Ok(Some(utxo1.clone())));

    mock_utxo_reader
        .expect_get_utxo()
        .withf(move |op| op == &outpoint2)
        .times(1)
        .returning(|_| Ok(None));

    let validator = DefaultTransactionValidator::new(Arc::new(mock_utxo_reader));

    let input1 = TransactionInput::new(create_test_outpoint(1, 0));
    let input2 = TransactionInput::new(create_test_outpoint(2, 0));
    let output = TransactionOutput::new(create_test_wallet_address(3), TransactionAmount::new(100));
    let tx = NonValidatedTransaction::new(
        vec![input1, input2],
        vec![output],
        DateTime::from_ms(1000000000),
    )
    .unwrap();

    let result = validator.pub_validate_inputs_unspent(&tx).await;
    assert!(result.is_err(), "Should fail when any UTXO not found");

    match result.unwrap_err() {
        AppError::TransactionValidation(TransactionValidationError::InputUtxoNotFound {
            ..
        }) => {}
        _ => panic!("Expected InputUtxoNotFound error"),
    }
}
