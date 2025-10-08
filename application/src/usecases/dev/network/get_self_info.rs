use common::error::AppError;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::types::network::{NetworkAddress, NetworkIdentityKeypair};
use std::sync::Arc;

#[derive(Clone)]
pub struct GetNetworkSelfInfoUseCase {
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetNetworkSelfInfoUseCase {
    pub fn new(
        cmd_tx: Arc<dyn CommandSender>,
        cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            cmd_tx,
            cmd_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetNetworkSelfInfoUseCaseResponse, AppError> {
        let (command, res_fut) = self.cmd_tx_res_factory.build_net_cmd_get_self_info();
        self.cmd_tx.send(command).await?;
        let (identity_key_pair, network_addresses) = res_fut.await?;
        let res = GetNetworkSelfInfoUseCaseResponse {
            identity_key_pair,
            network_addresses,
        };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GetNetworkSelfInfoUseCaseResponse {
    pub identity_key_pair: NetworkIdentityKeypair,
    pub network_addresses: Vec<NetworkAddress>,
}
