use crate::ext::AppErrorExtInfrastructure;
use common::config::network::NetworkConfig;
use common::error::AppError;
use domain::repos::network::NetworkRepository;
use domain::system::network::P2PNetworkHandle;
use domain::system::network::event::NetworkEvent;
use libp2p::gossipsub::IdentTopic;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct Libp2pNetworkHandle {
    #[allow(unused)]
    pub(super) config: NetworkConfig,
    #[allow(unused)]
    pub(super) network_repo: Arc<dyn NetworkRepository>,
    #[allow(unused)]
    pub(super) topic: IdentTopic,
    pub(super) events_tx: UnboundedSender<NetworkEvent>,
}

impl P2PNetworkHandle for Libp2pNetworkHandle {
    fn publish_network_event(&self, event: NetworkEvent) -> Result<(), AppError> {
        // TODO: success doesn't ensure delivery!
        self.events_tx.send(event).to_app_error()
    }
}
