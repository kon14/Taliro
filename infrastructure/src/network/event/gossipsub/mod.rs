use super::super::behavior::AppNetworkBehavior;
use common::{log_net_gs_error, log_net_kad_debug, log_net_kad_trace};
use domain::types::network::NetworkPeerId;
use domain::{
    encode::TryDecode,
    system::{
        network::event::GossipsubNetworkEvent,
        node::cmd::{CommandResponderFactory, CommandSender},
    },
};
use libp2p::{Swarm, gossipsub};
use std::sync::Arc;

pub(super) async fn handle_gossipsub_event(
    event: gossipsub::Event,
    swarm: &mut Swarm<AppNetworkBehavior>,
    cmd_tx: &Arc<dyn CommandSender>,
    cmd_tx_res_factory: &Arc<dyn CommandResponderFactory>,
) {
    #[allow(clippy::single_match)]
    match event {
        gossipsub::Event::Message { .. } => {
            handle_gossipsub_message_event(event, swarm, cmd_tx, cmd_tx_res_factory).await
        }
        _ => {}
    }
}

async fn handle_gossipsub_message_event(
    event: gossipsub::Event,
    swarm: &mut Swarm<AppNetworkBehavior>,
    cmd_tx: &Arc<dyn CommandSender>,
    cmd_tx_res_factory: &Arc<dyn CommandResponderFactory>,
) {
    let gossipsub::Event::Message {
        message,
        propagation_source,
        ..
    } = event
    else {
        return;
    };

    // Ignore the voices in our own head...
    if &propagation_source == swarm.local_peer_id() {
        return;
    }

    log_net_kad_trace!("Gossipsub event: {:?}", message);

    let event = match GossipsubNetworkEvent::try_decode(&message.data) {
        Ok(event) => event,
        Err(err) => {
            log_net_gs_error!(
                "Failed to decode GossipsubNetworkEvent from gossipsub message: {}",
                err
            );
            return;
        }
    };

    log_net_kad_debug!("Received gossipsub GossipsubNetworkEvent: {:?}", event);
    // Already a PeerId
    let peer_id = NetworkPeerId::_new_validated(
        propagation_source.to_bytes(),
        propagation_source.to_string(),
    );

    #[allow(clippy::single_match)]
    match event {
        GossipsubNetworkEvent::BroadcastNewBlock(block) => {
            let block = block.invalidate();
            let (command, res_fut) =
                cmd_tx_res_factory.build_p2p_cmd_receive_blocks(peer_id, vec![block]);
            let Ok(_) = cmd_tx.send(command).await else {
                log_net_gs_error!("Failed to send HandleNetworkBroadcastNewBlock command.");
                return;
            };
            if let Err(err) = res_fut.await {
                log_net_gs_error!("HandleNetworkBroadcastNewBlock command failed: {}", err);
            }
        }
    }
}
