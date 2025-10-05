use common::error::AppError;
use domain::entities::transaction::Utxo;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use std::sync::Arc;

#[derive(Clone)]
pub struct GetUtxosUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetUtxosUseCase {
    pub fn new(
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            bus_tx,
            bus_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetUtxosUseCaseResponse, AppError> {
        let (command, res_fut) = self.bus_tx_res_factory.build_cmd_get_utxos();
        self.bus_tx.send(command).await?;
        let utxos = res_fut.await?;
        let res = GetUtxosUseCaseResponse { utxos };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetUtxosUseCaseResponse {
    pub utxos: Vec<Utxo>,
}
