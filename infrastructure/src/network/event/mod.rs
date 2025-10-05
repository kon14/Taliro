mod gossipsub;
mod kademlia;
mod taliro;

use super::protocol::{TaliroProtocolRequest, TaliroProtocolResponse};
use crate::network::behavior::AppNetworkBehavior;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use gossipsub::handle_gossipsub_event;
use kademlia::handle_kademlia_event;
use std::sync::Arc;
use taliro::handle_taliro_event;

#[derive(Debug)]
pub(crate) enum AppNetworkEvent {
    Gossipsub(libp2p::gossipsub::Event),
    Kademlia(libp2p::kad::Event),
    Taliro(libp2p::request_response::Event<TaliroProtocolRequest, TaliroProtocolResponse>),
}

impl From<libp2p::gossipsub::Event> for AppNetworkEvent {
    fn from(event: libp2p::gossipsub::Event) -> Self {
        AppNetworkEvent::Gossipsub(event)
    }
}

impl From<libp2p::kad::Event> for AppNetworkEvent {
    fn from(event: libp2p::kad::Event) -> Self {
        AppNetworkEvent::Kademlia(event)
    }
}

impl From<libp2p::request_response::Event<TaliroProtocolRequest, TaliroProtocolResponse>>
    for AppNetworkEvent
{
    fn from(
        event: libp2p::request_response::Event<TaliroProtocolRequest, TaliroProtocolResponse>,
    ) -> Self {
        AppNetworkEvent::Taliro(event)
    }
}

impl AppNetworkEvent {
    pub(super) async fn handle_behavior_event(
        event: Self,
        swarm: &mut libp2p::Swarm<AppNetworkBehavior>,
        bus_tx: &Arc<dyn CommandSender>,
        bus_tx_res_factory: &Arc<dyn CommandResponderFactory>,
        network_repo: &Arc<dyn domain::repos::network::NetworkRepository>,
        termination_initiated: bool,
    ) {
        match event {
            AppNetworkEvent::Gossipsub(event) => {
                handle_gossipsub_event(event, swarm, bus_tx, bus_tx_res_factory).await;
            }
            AppNetworkEvent::Kademlia(event) => {
                handle_kademlia_event(event, swarm, network_repo, termination_initiated);
            }
            AppNetworkEvent::Taliro(event) => {
                handle_taliro_event(event, swarm, bus_tx, bus_tx_res_factory).await;
            }
        }
    }
}
