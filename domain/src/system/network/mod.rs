pub mod event;

use crate::system::node::bus::{CommandResponderFactory, CommandSender};
use async_trait::async_trait;
use common::error::AppError;
use event::NetworkEvent;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait P2PNetworkEngine: Debug {
    async fn connect(
        mut self: Box<Self>,
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Arc<dyn P2PNetworkHandle>, AppError>;
}

pub trait P2PNetworkHandle: Debug + Send + Sync {
    fn publish_network_event(&self, event: NetworkEvent) -> Result<(), AppError>;
}
