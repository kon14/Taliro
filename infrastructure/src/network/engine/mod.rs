use super::behavior::AppNetworkBehavior;
use super::event::AppNetworkEvent;
use super::handle::Libp2pNetworkHandle;
use super::protocol;
use crate::ext::*;
use crate::network::store::NetworkPeerStore;
use async_trait::async_trait;
use common::config::network::NetworkConfig;
use common::error::AppError;
use common::{
    log_net_debug, log_net_error, log_net_gs_error, log_net_info, log_net_taliro_error,
    log_net_taliro_trace, log_net_trace, log_net_warn,
};
use domain::encode::TryEncode;
use domain::repos::network::NetworkRepository;
use domain::system::network::event::{AddPeerResponse, NetworkEvent};
use domain::system::network::{P2PNetworkEngine, P2PNetworkHandle};
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use domain::types::network::{NetworkAddress, NetworkIdentityKeypair, NetworkPeerId};
use libp2p::core::transport::ListenerId;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{IdentTopic, Topic};
use libp2p::identity::Keypair;
use libp2p::kad;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId, Swarm, noise, tcp, yamux};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

pub struct Libp2pNetworkEngine {
    config: NetworkConfig,
    swarm: Swarm<AppNetworkBehavior>,
    network_repo: Arc<dyn NetworkRepository>,
    topic: IdentTopic,
    peer_store: Arc<NetworkPeerStore>,
    identity: NetworkIdentityKeypair,
    listener_id: ListenerId,
}

impl Libp2pNetworkEngine {
    const TALIRO_TOPIC: &'static str = "taliro";

    pub fn new(
        cfg: NetworkConfig,
        network_repo: Arc<dyn NetworkRepository>,
    ) -> Result<Self, AppError> {
        let topic = Topic::new(Self::TALIRO_TOPIC);

        let (key_pair, net_key_pair) = Self::get_identity(&cfg, network_repo.clone())?;
        let (listen_addr, peer_id) = Self::get_listen_address(&cfg, &key_pair)?;
        let peer_store = NetworkPeerStore::new(peer_id);

        if listen_addr.has_ephemeral_port() {
            log_net_warn!("Ephemeral port (tcp/0) used; peers won't auto-reconnect on restart!");
        }

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key_pair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .to_app_error()?
            .with_dns()
            .to_app_error()?
            .with_behaviour(|keys| {
                let behavior = AppNetworkBehavior::new(keys)?;
                Ok(behavior)
            })
            .to_app_error()?
            .build();

        swarm
            .behaviour_mut()
            .get_kademlia_mut()
            .set_mode(Some(kad::Mode::Server));

        let listener_id = swarm.listen_on(listen_addr).to_app_error()?;

        let network = Libp2pNetworkEngine {
            config: cfg,
            swarm,
            network_repo,
            topic,
            peer_store: Arc::new(peer_store),
            identity: net_key_pair,
            listener_id,
        };
        Ok(network)
    }

    fn handle_termination(swarm: &mut Swarm<AppNetworkBehavior>, listeners: &HashSet<ListenerId>) {
        log_net_info!("Shutdown signal received, cleaning up network...");

        // Disconnect all connected peers
        let connected_peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
        for peer_id in connected_peers {
            let _ = swarm.disconnect_peer_id(peer_id);
            log_net_info!("Disconnected from peer: {peer_id}");
        }

        // Remove all listeners to close TCP ports
        for listener in listeners {
            let active = swarm.remove_listener(*listener);
            if active {
                log_net_info!("Removing P2P listener: {listener}");
            }
        }

        log_net_info!("Network cleanup completed");
    }

    async fn handle_outgoing_event(
        swarm: &mut Swarm<AppNetworkBehavior>,
        topic: &IdentTopic,
        peer_store: &Arc<NetworkPeerStore>,
        identity: &NetworkIdentityKeypair,
        event: NetworkEvent,
    ) {
        #[allow(clippy::single_match)]
        match event {
            NetworkEvent::Gossipsub(event) => {
                let Ok(data) = event.try_encode() else {
                    log_net_gs_error!("Failed to encode GossipsubNetworkEvent for broadcasting.");
                    return;
                };
                let Ok(_) = swarm
                    .behaviour_mut()
                    .get_gossipsub_mut()
                    .publish(topic.clone(), data)
                else {
                    log_net_gs_error!("Failed to publish GossipsubNetworkEvent to gossipsub.");
                    return;
                };
            }
            NetworkEvent::Taliro(event) => {
                let Ok(peer_id) = event
                    .get_header()
                    .get_recipient_peer_id()
                    .try_into_libp2p_peer_id()
                else {
                    log_net_taliro_error!(
                        "Invalid or missing recipient peer ID in TaliroNetworkEvent header."
                    );
                    return;
                };
                let request = event.get_data().build_taliro_request();
                let request_id = swarm
                    .behaviour_mut()
                    .get_blockchain_mut()
                    .send_request(&peer_id, request);
                log_net_taliro_trace!("Sent Taliro request {request_id:?} to {peer_id}");
            }
            NetworkEvent::GetSelfInfo(responder) => {
                let network_addresses = peer_store.get_own_addresses().await;
                let _ = responder.send((identity.clone(), network_addresses));
            }
            NetworkEvent::GetPeers(responder) => {
                let network_addresses = peer_store.get_flat_addresses().await;
                let _ = responder.send(network_addresses);
            }
            NetworkEvent::AddPeer(address, responder) => {
                log_net_info!("Handling AddPeer event for address: {:?}", address);
                let Ok(addr) = address.clone().try_into_libp2p_addr() else {
                    let res = AddPeerResponse::InvalidAddress(address);
                    let _ = responder.send(res);
                    return;
                };

                let already_connected = peer_store.is_address_known(&address).await;
                if already_connected {
                    let res = AddPeerResponse::AlreadyConnected;
                    let _ = responder.send(res);
                    return;
                }

                let res = Swarm::dial(swarm, addr.clone()).map_or_else(
                    |err| {
                        log_net_error!("Failed to dial peer address {:?}: {}", addr, err);
                        AddPeerResponse::FailedToDialPeer
                    },
                    |_| AddPeerResponse::Pending,
                );
                let _ = responder.send(res);
            }
        }
    }

    async fn handle_incoming_event(
        swarm: &mut Swarm<AppNetworkBehavior>,
        network_repo: &Arc<dyn NetworkRepository>,
        bus_tx: &Arc<dyn CommandSender>,
        bus_tx_res_factory: &Arc<dyn CommandResponderFactory>,
        peer_store: &Arc<NetworkPeerStore>,
        active_listeners: &mut HashSet<ListenerId>,
        termination_initiated: bool,
    ) {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                let full_address = address.clone().with(Protocol::P2p(*swarm.local_peer_id()));
                log_net_info!("Listening on {full_address:?}");

                let Ok(network_address) = full_address.clone().try_into_domain_network_address()
                else {
                    log_net_error!("Failed to parse self address from multiaddr: {full_address:?}");
                    return;
                };
                peer_store.add_own_address(network_address).await;
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                log_net_info!("Connection established with {peer_id} at {endpoint:?}");
                let mut multiaddr = endpoint.get_remote_address().clone();
                multiaddr.set_peer_id(peer_id);

                if let Ok(net_addr) = multiaddr.clone().try_into_domain_network_address() {
                    let net_peer_id =
                        NetworkPeerId::new_unchecked(peer_id.to_bytes(), peer_id.to_string());
                    let is_new_connection = peer_store
                        .add_peer_address(net_peer_id, net_addr.clone())
                        .await;

                    #[allow(clippy::collapsible_if)]
                    if is_new_connection {
                        if let Err(err) = network_repo.insert_peer_address(net_addr) {
                            log_net_error!(
                                "Failed to insert peer address into NetworkRepository! Error: {err}"
                            );
                        }
                    }
                } else {
                    log_net_error!("Failed to parse peer address from multiaddr: {multiaddr:?}");
                    log_net_debug!("NetworkPeerStore out of sync!");
                }

                // TODO: move below logic to Domain
                // TODO: defer syncing until after Node has started.
                let request = protocol::TaliroProtocolRequest::GetBlockchainTip;
                let request_id = swarm
                    .behaviour_mut()
                    .get_blockchain_mut()
                    .send_request(&peer_id, request);
                log_net_trace!("Sent Taliro request {request_id:?} to {peer_id}");
            }
            SwarmEvent::ConnectionClosed {
                peer_id, endpoint, ..
            } => {
                log_net_warn!("Connection dropped for {peer_id} at {endpoint:?}");
                let mut multiaddr = endpoint.get_remote_address().clone();
                multiaddr.set_peer_id(peer_id);
                if let Ok(net_addr) = multiaddr.clone().try_into_domain_network_address() {
                    let net_peer_id =
                        NetworkPeerId::new_unchecked(peer_id.to_bytes(), peer_id.to_string());
                    peer_store.remove_peer_address(net_peer_id, &net_addr).await;
                } else {
                    log_net_error!("Failed to parse peer address from multiaddr: {multiaddr:?}");
                    log_net_debug!("NetworkPeerStore out of sync!");
                }
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                log_net_warn!("Outgoing connection failed for {peer_id:?}. | Error: {error}");
            }
            SwarmEvent::ListenerClosed { listener_id, .. } => {
                log_net_info!("P2P listener closed: {listener_id}");
                active_listeners.remove(&listener_id);
            }
            SwarmEvent::Behaviour(behavior_event) => {
                AppNetworkEvent::handle_behavior_event(
                    behavior_event,
                    swarm,
                    bus_tx,
                    bus_tx_res_factory,
                    network_repo,
                    termination_initiated,
                )
                .await
            }
            _ => {}
        }
    }

    fn get_identity(
        cfg: &NetworkConfig,
        network_repo: Arc<dyn NetworkRepository>,
    ) -> Result<(Keypair, NetworkIdentityKeypair), AppError> {
        // Try retrieving identity from config.
        let mut identity = cfg
            .identity_key_pair
            .as_ref()
            .map(|encoded_key_pair| NetworkIdentityKeypair::from_base64(encoded_key_pair.clone()));

        // Try retrieving identity from repo.
        if identity.is_none() {
            identity = network_repo.get_identity_keys()?;
        }

        // Try using existing identity.
        if let Some(net_key_pair) = identity {
            let key_pair = net_key_pair.clone().try_into_libp2p_keypair()?;
            return Ok((key_pair, net_key_pair));
        };

        // Fall back to generating a new identity.
        let key_pair = Keypair::generate_ed25519();
        let net_key_pair = key_pair.clone().try_into_domain_keypair()?;

        // Persist identity.
        network_repo.insert_identity_keys(net_key_pair.clone())?;
        Ok((key_pair, net_key_pair))
    }

    fn get_listen_address(
        cfg: &NetworkConfig,
        identity: &Keypair,
    ) -> Result<(Multiaddr, NetworkPeerId), AppError> {
        let mut multiaddr = cfg
            .listen_address
            .parse::<Multiaddr>()
            .map_err(|err| AppError::internal(format!("Invalid multiaddr format: {}", err)))?;
        let peer_id = PeerId::from(identity.public());
        multiaddr.set_peer_id(peer_id);
        let net_peer_id = NetworkPeerId::new_unchecked(peer_id.to_bytes(), peer_id.to_string());
        Ok((multiaddr, net_peer_id))
    }

    fn get_peer_addresses(&self) -> Result<HashSet<NetworkAddress>, AppError> {
        let init_peer_addresses = self
            .config
            .init_peers
            .iter()
            .map(|addr| NetworkAddress::try_from_str(addr)) // TODO: early validation (move into cfg)
            .collect::<Result<HashSet<_>, AppError>>()?;
        let mut peer_addresses = self.network_repo.get_peer_addresses()?;
        peer_addresses.extend(init_peer_addresses);
        let own_peer_id = self.peer_store.get_own_peer_id();
        let peer_addresses = peer_addresses
            .into_iter()
            .filter(|addr| addr.get_peer_id().as_ref() != Some(own_peer_id))
            .collect();
        Ok(peer_addresses)
    }

    fn spawn_network_event_handler_worker_task(
        self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
        mut events_rx: tokio::sync::mpsc::UnboundedReceiver<NetworkEvent>,
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
    ) {
        tokio::spawn({
            let mut swarm = self.swarm;
            let network_repo = self.network_repo;
            let topic = self.topic;
            let peer_store = self.peer_store;
            let identity = self.identity;
            let mut active_listeners: HashSet<ListenerId> =
                [self.listener_id].into_iter().collect();
            let mut termination_initiated = false;
            async move {
                loop {
                    if active_listeners.is_empty() {
                        break;
                    }
                    tokio::select! {
                        // Handle shutdown signal
                        _ = shutdown_rx.recv(), if !termination_initiated => {
                            termination_initiated = true;
                            log_net_warn!("term init: {:?}", termination_initiated);
                            Self::handle_termination(&mut swarm, &active_listeners);
                        }

                        // Handle outgoing events
                        Some(event) = events_rx.recv() => Self::handle_outgoing_event(
                            &mut swarm,
                            &topic,
                            &peer_store,
                            &identity,
                            event,
                        ).await,

                        // Handle incoming events
                        _ = Self::handle_incoming_event(
                            &mut swarm,
                            &network_repo,
                            &bus_tx,
                            &bus_tx_res_factory,
                            &peer_store,
                            &mut active_listeners,
                            termination_initiated,
                        ) => {},
                    }
                }

                log_net_info!("Network event handler worker task exiting...");
            }
        });
    }
}

#[async_trait]
impl P2PNetworkEngine for Libp2pNetworkEngine {
    async fn connect(
        mut self: Box<Self>,
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Arc<dyn P2PNetworkHandle>, AppError> {
        let peer_addresses = self
            .get_peer_addresses()?
            .into_iter()
            .map(|addr| addr.try_into_libp2p_addr())
            .collect::<Result<Vec<_>, AppError>>()?;
        for addr in &peer_addresses {
            Swarm::dial(&mut self.swarm, addr.clone()).unwrap_or_else(|err| {
                log_net_error!("Failed to dial initial peer address {:?}: {}", addr, err);
            });
        }

        let (events_tx, events_rx) = tokio::sync::mpsc::unbounded_channel::<NetworkEvent>();

        self.swarm
            .behaviour_mut()
            .get_gossipsub_mut()
            .subscribe(&self.topic)
            .to_app_error()?;

        let network = Libp2pNetworkHandle {
            config: self.config.clone(),
            network_repo: self.network_repo.clone(),
            topic: self.topic.clone(),
            events_tx,
        };

        self.spawn_network_event_handler_worker_task(
            shutdown_rx,
            events_rx,
            bus_tx,
            bus_tx_res_factory,
        );

        Ok(Arc::new(network))
    }
}

impl Debug for Libp2pNetworkEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Libp2pNetwork")
            .field("topic", &self.topic)
            .finish()
    }
}
