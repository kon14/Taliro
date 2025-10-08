use common::error::AppError;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::types::network::NetworkAddress;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetNetworkPeersUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetNetworkPeersUseCase {
    pub fn new(
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            cmd_tx,
            cmd_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetNetworkPeersUseCaseResponse, AppError> {
        let (command, res_fut) = self.cmd_tx_res_factory.build_net_cmd_get_peers();
        self.cmd_tx.send(command).await?;
        let peers = res_fut.await?;
        let res = GetNetworkPeersUseCaseResponse { peers };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetNetworkPeersUseCaseResponse {
    pub peers: Vec<NetworkAddress>,
}
