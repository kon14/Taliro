use common::error::AppError;
use common::params::PaginationParams;
use domain::entities::transaction::Transaction;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use std::sync::Arc;

#[derive(Clone)]
pub struct GetMempoolTransactionsUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetMempoolTransactionsUseCase {
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
        pagination: PaginationParams,
    ) -> Result<GetMempoolTransactionsUseCaseResponse, AppError> {
        let (command, res_fut) = self
            .cmd_tx_res_factory
            .build_mp_get_paginated_transactions(pagination);
        self.cmd_tx.send(command).await?;
        let (transactions, count) = res_fut.await?;
        let res = GetMempoolTransactionsUseCaseResponse {
            transactions,
            count,
        };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetMempoolTransactionsUseCaseResponse {
    pub transactions: Vec<Transaction>,
    pub count: usize,
}
