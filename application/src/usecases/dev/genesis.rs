use common::error::AppError;
use domain::genesis::config::GenesisConfig;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use std::sync::Arc;

#[derive(Clone)]
pub struct InitiateGenesisUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl InitiateGenesisUseCase {
    pub fn new(
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            bus_tx,
            bus_tx_res_factory,
        }
    }

    pub async fn execute(&self, request: InitiateGenesisUseCaseRequest) -> Result<(), AppError> {
        let genesis_cfg = request.genesis_cfg;
        let (command, res_fut) = self
            .bus_tx_res_factory
            .build_blk_cmd_init_genesis(genesis_cfg);
        self.bus_tx.send(command).await?;

        // Note: this doesn't currently await blockchain tip update...
        res_fut.await
    }
}

#[derive(Debug)]
pub struct InitiateGenesisUseCaseRequest {
    pub genesis_cfg: GenesisConfig,
}
