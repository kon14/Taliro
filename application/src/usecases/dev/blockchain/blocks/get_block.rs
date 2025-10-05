use common::error::AppError;
use domain::entities::block::Block;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::hash::Hash;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetBlockchainBlockUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetBlockchainBlockUseCase {
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
        request: GetBlockchainBlockUseCaseRequest,
    ) -> Result<GetBlockchainBlockUseCaseResponse, AppError> {
        let block_hash = request.block_hash;
        let (command, res_fut) = self.bus_tx_res_factory.build_blk_cmd_get_block(block_hash);
        self.bus_tx.send(command).await?;
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
