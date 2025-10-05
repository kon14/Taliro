use domain::system::network::event::AddPeerResponse;
use domain::types::network::{NetworkAddress, NetworkIdentityKeypair};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[schema(title = "NetworkAddress")]
pub(crate) struct NetworkAddressPresentationDto {
    address: String,
    peer_id: Option<String>,
}

impl From<NetworkAddress> for NetworkAddressPresentationDto {
    fn from(addr: NetworkAddress) -> Self {
        Self {
            address: addr.get_address_str(),
            peer_id: addr
                .get_peer_id()
                .map(|peer_id| peer_id.as_str().to_string()),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "status", content = "data")]
#[schema(title = "AddPeerResponseStatus")]
pub(crate) enum AddPeerResponseStatusPresentationDto {
    Pending,
    AlreadyConnected,
    InvalidAddress(NetworkAddressPresentationDto),
    FailedToDialPeer,
}

impl From<AddPeerResponse> for AddPeerResponseStatusPresentationDto {
    fn from(res: AddPeerResponse) -> Self {
        match res {
            AddPeerResponse::Pending => Self::Pending,
            AddPeerResponse::AlreadyConnected => Self::AlreadyConnected,
            AddPeerResponse::InvalidAddress(addr) => Self::InvalidAddress(addr.into()),
            AddPeerResponse::FailedToDialPeer => Self::FailedToDialPeer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "NetworkIdentityKeypair")]
pub(crate) struct NetworkIdentityKeypairPresentationDto(String);

impl From<NetworkIdentityKeypair> for NetworkIdentityKeypairPresentationDto {
    fn from(keypair: NetworkIdentityKeypair) -> Self {
        Self(keypair.as_base64())
    }
}
