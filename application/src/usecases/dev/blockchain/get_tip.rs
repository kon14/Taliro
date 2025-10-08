use common::error::AppError;
use domain::entities::block::BlockHeight;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::types::hash::Hash;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetBlockchainTipInfoUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetBlockchainTipInfoUseCase {
    pub fn new(
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            cmd_tx,
            cmd_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetBlockchainTipInfoUseCaseResponse, AppError> {
        let (command, res_fut) = self.cmd_tx_res_factory.build_blk_cmd_get_tip_info();
        self.cmd_tx.send(command).await?;
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
