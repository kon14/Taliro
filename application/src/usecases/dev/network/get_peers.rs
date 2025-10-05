use common::error::AppError;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::network::NetworkAddress;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetNetworkPeersUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetNetworkPeersUseCase {
    pub fn new(
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            bus_tx,
            bus_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetNetworkPeersUseCaseResponse, AppError> {
        let (command, res_fut) = self.bus_tx_res_factory.build_net_cmd_get_peers();
        self.bus_tx.send(command).await?;
        let peers = res_fut.await?;
        let res = GetNetworkPeersUseCaseResponse { peers };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetNetworkPeersUseCaseResponse {
    pub peers: Vec<NetworkAddress>,
}
