use common::error::AppError;
use domain::entities::block::BlockHeight;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::hash::Hash;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetBlockchainTipInfoUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetBlockchainTipInfoUseCase {
    pub fn new(
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            bus_tx,
            bus_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetBlockchainTipInfoUseCaseResponse, AppError> {
        let (command, res_fut) = self.bus_tx_res_factory.build_blk_cmd_get_tip_info();
        self.bus_tx.send(command).await?;
        let block = res_fut
            .await?
            .map(|(hash, height)| GetBlockchainTipInfoUseCaseResponseBlock { hash, height });
        let res = GetBlockchainTipInfoUseCaseResponse { block };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetBlockchainTipInfoUseCaseResponse {
    pub block: Option<GetBlockchainTipInfoUseCaseResponseBlock>,
}

#[derive(Debug)]
pub struct GetBlockchainTipInfoUseCaseResponseBlock {
    pub hash: Hash,
    pub height: BlockHeight,
}
