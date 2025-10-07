use common::error::AppError;
use domain::system::network::event::AddPeerResponse;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::network::NetworkAddress;
use std::sync::Arc;

#[derive(Clone)]
pub struct AddNetworkPeerUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl AddNetworkPeerUseCase {
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
        network_address: NetworkAddress,
    ) -> Result<AddPeerResponse, AppError> {
        let (command, res_fut) = self
            .bus_tx_res_factory
            .build_net_cmd_add_peer(network_address);
        self.bus_tx.send(command).await?;
        let res = res_fut.await?;
        Ok(res)
    }
}
