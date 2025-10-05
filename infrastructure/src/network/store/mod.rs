use domain::types::network::{NetworkAddress, NetworkPeerId};
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

pub(crate) struct NetworkPeerStore {
    peer_id: NetworkPeerId,
    own_addresses: Mutex<HashSet<NetworkAddress>>,
    connected_peers: Mutex<HashMap<NetworkPeerId, HashSet<NetworkAddress>>>,
}

impl NetworkPeerStore {
    pub(crate) fn new(peer_id: NetworkPeerId) -> Self {
        Self {
            peer_id,
            own_addresses: Mutex::new(HashSet::new()),
            connected_peers: Mutex::new(HashMap::default()),
        }
    }

    pub(crate) fn get_own_peer_id(&self) -> &NetworkPeerId {
        &self.peer_id
    }

    pub(crate) async fn get_own_addresses(&self) -> Vec<NetworkAddress> {
        let addrs = self.own_addresses.lock().await;
        addrs.iter().cloned().collect()
    }

    pub(crate) async fn is_address_known(&self, address: &NetworkAddress) -> bool {
        let peers = self.connected_peers.lock().await;
        peers.values().any(|addrs| addrs.contains(address))
    }

    pub(crate) async fn get_flat_addresses(&self) -> Vec<NetworkAddress> {
        let peers = self.connected_peers.lock().await;
        peers
            .values()
            .flat_map(|addrs| addrs.iter().cloned())
            .collect()
    }

    /// Adds a peer address.<br />
    /// Returns `true` if the address was newly registered, `false` if it was already known.
    pub(crate) async fn add_peer_address(
        &self,
        peer_id: NetworkPeerId,
        address: NetworkAddress,
    ) -> bool {
        let mut peers = self.connected_peers.lock().await;
        peers.entry(peer_id).or_default().insert(address)
    }

    /// Adds an own address.<br />
    /// Returns `true` if the address was newly registered, `false` if it was already known.
    pub(crate) async fn add_own_address(&self, address: NetworkAddress) {
        let mut addrs = self.own_addresses.lock().await;
        addrs.insert(address);
    }

    /// Removes a peer address.<br />
    /// Returns `true` if the address was removed, `false` if it wasn't found.
    pub(crate) async fn remove_peer_address(
        &self,
        peer_id: NetworkPeerId,
        address: &NetworkAddress,
    ) -> bool {
        let mut peers = self.connected_peers.lock().await;
        if let Some(addrs) = peers.get_mut(&peer_id) {
            let found = addrs.remove(address);
            if addrs.is_empty() {
                peers.remove(&peer_id);
            }
            return found;
        }
        false
    }
}
