use common::error::AppError;
use common::params::PaginationParams;
use domain::entities::transaction::Transaction;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use std::sync::Arc;

#[derive(Clone)]
pub struct GetMempoolTransactionsUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetMempoolTransactionsUseCase {
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
        pagination: PaginationParams,
    ) -> Result<GetMempoolTransactionsUseCaseResponse, AppError> {
        let (command, res_fut) = self
            .bus_tx_res_factory
            .build_mp_get_paginated_transactions(pagination);
        self.bus_tx.send(command).await?;
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
