use super::super::behavior::AppNetworkBehavior;
use common::{log_net_kad_debug, log_net_kad_error, log_net_kad_trace};
use libp2p::multiaddr::Protocol;
use libp2p::{Swarm, kad};
use std::sync::Arc;

pub(super) fn handle_kademlia_event(
    event: kad::Event,
    swarm: &mut Swarm<AppNetworkBehavior>,
    network_repo: &Arc<dyn domain::repos::network::NetworkRepository>,
    termination_initiated: bool,
    net_entity_validator: &Arc<dyn domain::system::network::validator::NetworkEntityValidator>,
) {
    log_net_kad_trace!("Kademlia event: {:?}", event);

    #[allow(clippy::single_match)]
    match event {
        kad::Event::RoutingUpdated {
            peer,
            addresses,
            is_new_peer,
            ..
        } => {
            if termination_initiated {
                log_net_kad_debug!(
                    "Ignoring Kademlia routing update for peer ({peer}). Node currently terminating."
                );
                return;
            }

            for base_addr in addresses.iter() {
                let mut full_addr = base_addr.clone();
                full_addr.push(Protocol::P2p(peer.into()));
                log_net_kad_trace!(
                    "P2PNetwork.handle_incoming_event() | Kademlia | RoutingUpdated | Address: {:?}",
                    full_addr
                );
                if is_new_peer {
                    let net_addr = match net_entity_validator
                        .validate_address(full_addr.to_string())
                    {
                        Ok(addr) => addr,
                        Err(err) => {
                            log_net_kad_error!(
                                "RoutingUpdated - Failed to validate peer address! | Error: {err}"
                            );
                            continue;
                        }
                    };
                    let Ok(_) = network_repo.insert_peer_address(net_addr) else {
                        log_net_kad_error!(
                            "RoutingUpdated - Failed to insert new peer address into repository."
                        );
                        continue;
                    };
                }

                // TODO: This may return Failure or Pending...
                swarm
                    .behaviour_mut()
                    .get_kademlia_mut()
                    .add_address(&peer, base_addr.clone());

                let Ok(_) = swarm.dial(base_addr.clone()) else {
                    log_net_kad_error!("RoutingUpdated - Failed to dial new peer address.");
                    continue;
                };
                let Ok(_) = swarm.behaviour_mut().get_kademlia_mut().bootstrap() else {
                    log_net_kad_error!(
                        "RoutingUpdated - Failed to bootstrap Kademlia after adding new peer."
                    );
                    continue;
                };
            }
        }
        _ => {}
    }
}
