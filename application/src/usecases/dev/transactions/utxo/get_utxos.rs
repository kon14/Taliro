use common::error::AppError;
use domain::entities::transaction::Utxo;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use std::sync::Arc;

#[derive(Clone)]
pub struct GetUtxosUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetUtxosUseCase {
    pub fn new(
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            cmd_tx,
            cmd_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetUtxosUseCaseResponse, AppError> {
        let (command, res_fut) = self.cmd_tx_res_factory.build_utxo_cmd_get_utxos();
        self.cmd_tx.send(command).await?;
        let utxos = res_fut.await?;
        let res = GetUtxosUseCaseResponse { utxos };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetUtxosUseCaseResponse {
    pub utxos: Vec<Utxo>,
}
