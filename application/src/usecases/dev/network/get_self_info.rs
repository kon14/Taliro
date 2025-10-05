use common::error::AppError;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::network::{NetworkAddress, NetworkIdentityKeypair};
use std::sync::Arc;

#[derive(Clone)]
pub struct GetNetworkSelfInfoUseCase {
    bus_tx: Arc<dyn CommandSender>,
    bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
}

impl GetNetworkSelfInfoUseCase {
    pub fn new(
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) -> Self {
        Self {
            bus_tx,
            bus_tx_res_factory,
        }
    }

    pub async fn execute(&self) -> Result<GetNetworkSelfInfoUseCaseResponse, AppError> {
        let (command, res_fut) = self.bus_tx_res_factory.build_net_cmd_get_self_info();
        self.bus_tx.send(command).await?;
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
