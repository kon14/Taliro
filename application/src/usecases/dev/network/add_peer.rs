use common::error::AppError;
use domain::system::network::event::AddPeerResponse;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::types::network::NetworkAddress;
use std::sync::Arc;

#[derive(Clone)]
pub struct AddNetworkPeerUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl AddNetworkPeerUseCase {
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
        network_address: NetworkAddress,
    ) -> Result<AddPeerResponse, AppError> {
        let (command, res_fut) = self
            .cmd_tx_res_factory
            .build_net_cmd_add_peer(network_address);
        self.cmd_tx.send(command).await?;
        let res = res_fut.await?;
        Ok(res)
    }
}
