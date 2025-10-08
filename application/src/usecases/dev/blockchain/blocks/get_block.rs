use common::error::AppError;
use domain::entities::block::Block;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::types::hash::Hash;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetBlockchainBlockUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetBlockchainBlockUseCase {
    pub fn new(
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            cmd_tx,
            cmd_tx_res_factory,
        }
    }

    pub async fn execute(
        &self,
        request: GetBlockchainBlockUseCaseRequest,
    ) -> Result<GetBlockchainBlockUseCaseResponse, AppError> {
        let block_hash = request.block_hash;
        let (command, res_fut) = self.cmd_tx_res_factory.build_blk_cmd_get_block(block_hash);
        self.cmd_tx.send(command).await?;
        let block = res_fut.await?;
        let res = GetBlockchainBlockUseCaseResponse { block };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetBlockchainBlockUseCaseRequest {
    pub block_hash: Hash,
}

#[derive(Debug)]
pub struct GetBlockchainBlockUseCaseResponse {
    pub block: Option<Block>,
}
