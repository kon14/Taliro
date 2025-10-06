use common::error::AppError;
use domain::entities::transaction::{
    NonValidatedTransaction, Transaction, TransactionAmount, TransactionInput, TransactionOutPoint,
    TransactionOutput, Utxo,
};
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::sign::PrivateKey;
use domain::types::time::DateTime;
use domain::types::wallet::WalletAddress;
use std::sync::Arc;

#[derive(Clone)]
pub struct PlaceMempoolTransactionUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl PlaceMempoolTransactionUseCase {
    pub fn new(
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            bus_tx,
            bus_tx_res_factory,
        }
    }

    pub async fn execute(
        &self,
        request: PlaceMempoolTransactionUseCaseRequest,
    ) -> Result<PlaceMempoolTransactionUseCaseResponse, AppError> {
        // Build transaction inputs
        let utxos = self.get_utxos(request.consumed_outpoints).await?;
        let input_amount = self.get_input_amount(&utxos)?;
        let inputs = self.build_inputs(utxos);

        // Build transaction outputs
        // Assuming no tx fee, private key passed in for dev convenience...
        let output_amount = request.amount;
        let sender_public_key = request.sender_private_key.get_public_key();
        let sender_address = (&sender_public_key).into();
        let outputs = self.build_outputs(
            sender_address,
            request.recipient_wallet_address,
            input_amount,
            output_amount,
        )?;

        // Build transaction (pre-validation)
        let tx = NonValidatedTransaction::new(inputs, outputs, DateTime::now())?;

        let (command, res_fut) = self.bus_tx_res_factory.build_mp_cmd_place_transaction(tx);
        self.bus_tx.send(command).await?;
        let transaction = res_fut.await?;
        let res = PlaceMempoolTransactionUseCaseResponse { transaction };
        Ok(res)
    }

    async fn get_utxos(&self, outpoints: Vec<TransactionOutPoint>) -> Result<Vec<Utxo>, AppError> {
        let expected_len = outpoints.len();

        let (command, res_fut) = self
            .bus_tx_res_factory
            .build_blk_get_utxos_by_outpoints(outpoints);
        self.bus_tx.send(command).await?;
        let utxos = res_fut.await?;

        if utxos.len() != expected_len {
            return Err(AppError::internal(format!(
                "Utxo count mismatch! Expected {} utxo(s), but only found {}.",
                expected_len,
                utxos.len(),
            )));
        }

        Ok(utxos)
    }

    fn get_input_amount(&self, utxos: &[Utxo]) -> Result<TransactionAmount, AppError> {
        utxos
            .iter()
            .try_fold(TransactionAmount::new(0), |mut acc, utxo| {
                acc.checked_add_assign(utxo.get_output().get_amount())?;
                Ok(acc)
            })
    }

    fn build_inputs(&self, utxos: Vec<Utxo>) -> Vec<TransactionInput> {
        utxos
            .into_iter()
            .map(|outpoint| {
                let prev_outpoint = outpoint.get_outpoint().clone();
                TransactionInput::new(prev_outpoint)
            })
            .collect()
    }

    fn build_outputs(
        &self,
        sender_address: WalletAddress,
        recipient_address: WalletAddress,
        input_amount: TransactionAmount,
        output_amount: TransactionAmount,
    ) -> Result<Vec<TransactionOutput>, AppError> {
        let change_amount = input_amount
            .checked_sub(output_amount)
            .ok_or_else(|| AppError::bad_request("Input amount is less than output amount!"))?;

        let primary_output = TransactionOutput::new(recipient_address, output_amount);

        let change_output = (change_amount > TransactionAmount::new(0))
            .then(|| TransactionOutput::new(sender_address, change_amount));

        let mut outputs = vec![primary_output];
        outputs.extend(change_output);
        Ok(outputs)
    }
}

#[derive(Debug)]
pub struct PlaceMempoolTransactionUseCaseRequest {
    pub sender_private_key: PrivateKey, // purely for dev convenience...
    pub recipient_wallet_address: WalletAddress,
    pub amount: TransactionAmount,
    pub consumed_outpoints: Vec<TransactionOutPoint>,
}

#[derive(Debug)]
pub struct PlaceMempoolTransactionUseCaseResponse {
    pub transaction: Transaction,
}
