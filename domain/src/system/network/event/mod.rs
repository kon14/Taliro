mod gossipsub;
mod taliro;

use crate::types::network::{NetworkAddress, NetworkIdentityKeypair};
pub use gossipsub::GossipsubNetworkEvent;
pub use taliro::{TaliroNetworkData, TaliroNetworkEvent, TaliroNetworkEventHeader};

#[derive(Debug)]
pub enum NetworkEvent {
    Gossipsub(GossipsubNetworkEvent),
    Taliro(TaliroNetworkEvent),
    GetSelfInfo(tokio::sync::oneshot::Sender<(NetworkIdentityKeypair, Vec<NetworkAddress>)>),
    GetPeers(tokio::sync::oneshot::Sender<Vec<NetworkAddress>>),
    AddPeer(
        NetworkAddress,
        tokio::sync::oneshot::Sender<AddPeerResponse>,
    ),
}

#[derive(Debug)]
pub enum AddPeerResponse {
    Pending,
    AlreadyConnected,
    InvalidAddress(NetworkAddress),
    FailedToDialPeer,
}
