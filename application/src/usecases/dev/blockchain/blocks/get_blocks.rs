use common::error::AppError;
use domain::entities::block::{Block, BlockHeight};
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use std::ops::RangeInclusive;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetBlockchainBlocksByHeightRangeUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetBlockchainBlocksByHeightRangeUseCase {
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
        request: GetBlockchainBlocksByHeightRangeUseCaseRequest,
    ) -> Result<GetBlockchainBlocksByHeightRangeUseCaseResponse, AppError> {
        let height_range = request.height_range;
        let (command, res_fut) = self
            .bus_tx_res_factory
            .build_blk_cmd_get_blocks_by_height_range(height_range);
        self.bus_tx.send(command).await?;
        let blocks = res_fut.await?;
        let res = GetBlockchainBlocksByHeightRangeUseCaseResponse { blocks };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetBlockchainBlocksByHeightRangeUseCaseRequest {
    pub height_range: RangeInclusive<BlockHeight>,
}

#[derive(Debug)]
pub struct GetBlockchainBlocksByHeightRangeUseCaseResponse {
    pub blocks: Vec<Block>,
}
