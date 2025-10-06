use common::error::AppError;
use domain::entities::block::{Block, BlockDifficultyTarget, BlockTemplate};
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::hash::Hash;
use std::sync::Arc;

#[derive(Clone)]
pub struct AdHocMineBlockUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl AdHocMineBlockUseCase {
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
        request: AdHocMineBlockUseCaseRequest,
    ) -> Result<AdHocMineBlockUseCaseResponse, AppError> {
        let (command, res_fut) = self.bus_tx_res_factory.build_blk_cmd_get_tip_info();
        self.bus_tx.send(command).await?;
        let Some(tip_info) = res_fut.await? else {
            return Err(AppError::precondition_failed(
                "Blockchain not bootstrapped yet! Please initiate genesis.",
            ));
        };

        let tx_hashes = request.transaction_hashes;
        let (command, res_fut) = self
            .bus_tx_res_factory
            .build_mp_cmd_get_transactions_by_hashes(tx_hashes);
        self.bus_tx.send(command).await?;
        let transactions = res_fut.await?;
        let transactions = transactions.into_iter().map(|tx| tx.invalidate()).collect();

        let (command, res_fut) = self.bus_tx_res_factory.build_blk_cmd_get_block(tip_info.0);
        self.bus_tx.send(command).await?;
        let tip_block = res_fut
            .await?
            .ok_or(AppError::internal("Tip block not found!"))?;

        let block_tpl =
            BlockTemplate::new(&tip_block, transactions, BlockDifficultyTarget::_new_stub());

        let (command, res_fut) = self
            .bus_tx_res_factory
            .build_blk_cmd_handle_mine_block(block_tpl);
        self.bus_tx.send(command).await?;
        let block = res_fut.await?;

        let res = AdHocMineBlockUseCaseResponse { block };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct AdHocMineBlockUseCaseRequest {
    pub transaction_hashes: Vec<Hash>,
}

#[derive(Debug)]
pub struct AdHocMineBlockUseCaseResponse {
    pub block: Block,
}
