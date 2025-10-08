use common::error::AppError;
use domain::genesis::config::GenesisConfig;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use std::sync::Arc;

#[derive(Clone)]
pub struct InitiateGenesisUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl InitiateGenesisUseCase {
    pub fn new(
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            cmd_tx,
            cmd_tx_res_factory,
        }
    }

    pub async fn execute(&self, request: InitiateGenesisUseCaseRequest) -> Result<(), AppError> {
        let genesis_cfg = request.genesis_cfg;
        let (command, res_fut) = self
            .cmd_tx_res_factory
            .build_blk_cmd_init_genesis(genesis_cfg);
        self.cmd_tx.send(command).await?;

        // Note: this doesn't currently await blockchain tip update...
        res_fut.await
    }
}

#[derive(Debug)]
pub struct InitiateGenesisUseCaseRequest {
    pub genesis_cfg: GenesisConfig,
}
