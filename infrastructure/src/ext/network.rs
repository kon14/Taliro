use super::error::AppErrorExtInfrastructure;
use common::error::AppError;
use domain::types::network::{NetworkAddress, NetworkPeerId};
use libp2p::multiaddr::Protocol;

pub(crate) trait TryInfoLibp2pAddressExtInfrastructure {
    fn try_into_libp2p_addr(self) -> Result<libp2p::Multiaddr, AppError>;
}

impl TryInfoLibp2pAddressExtInfrastructure for NetworkAddress {
    fn try_into_libp2p_addr(self) -> Result<libp2p::Multiaddr, AppError> {
        // self.as_bytes().try_into().map_err(|err| AppError::internal(format!("Invalid network address: {}", err)))
        // TODO: that^ was UTF-8... consider moving Multiaddr to domain (early validation)
        let str_bytes = self.get_address_bytes();
        let s = std::str::from_utf8(&str_bytes).map_err(|err| {
            AppError::internal(format!("Invalid UTF-8 in network address: {}", err))
        })?;

        s.parse::<libp2p::Multiaddr>()
            .map_err(|err| AppError::internal(format!("Invalid multiaddr format: {}", err)))
    }
}

pub(crate) trait TryIntoLibp2pKeypairExtInfrastructure {
    fn try_into_libp2p_keypair(self) -> Result<libp2p::identity::Keypair, AppError>;
}

impl TryIntoLibp2pKeypairExtInfrastructure for domain::types::network::NetworkIdentityKeypair {
    fn try_into_libp2p_keypair(self) -> Result<libp2p::identity::Keypair, AppError> {
        let encoded = self.as_proto_bytes()?;
        let identity =
            libp2p::identity::Keypair::from_protobuf_encoding(&encoded).to_app_error()?;
        Ok(identity)
    }
}

pub(crate) trait TryIntoDomainKeypairExtInfrastructure {
    fn try_into_domain_keypair(
        self,
    ) -> Result<domain::types::network::NetworkIdentityKeypair, AppError>;
}

impl TryIntoDomainKeypairExtInfrastructure for libp2p::identity::Keypair {
    fn try_into_domain_keypair(
        self,
    ) -> Result<domain::types::network::NetworkIdentityKeypair, AppError> {
        let bytes = self.to_protobuf_encoding().to_app_error()?;
        let identity = domain::types::network::NetworkIdentityKeypair::from_proto_bytes(bytes);
        Ok(identity)
    }
}

pub(crate) trait TryIntoDomainNetworkAddressExtInfrastructure {
    fn try_into_domain_network_address(self) -> Result<NetworkAddress, AppError>;
}

impl TryIntoDomainNetworkAddressExtInfrastructure for libp2p::Multiaddr {
    fn try_into_domain_network_address(self) -> Result<NetworkAddress, AppError> {
        let bytes = self.to_string().into_bytes();
        let peer_id = self.iter().find_map(|proto| match proto {
            Protocol::P2p(peer_id) => Some(NetworkPeerId::new_unchecked(
                peer_id.to_bytes(),
                peer_id.to_string(),
            )),
            _ => None,
        });
        let address = NetworkAddress::new_unchecked(bytes, self.to_string(), peer_id);
        Ok(address)
    }
}

pub(crate) trait TryFromStrDomainNetworkAddressExtInfrastructure {
    fn try_from_str(s: &str) -> Result<NetworkAddress, AppError>;
}

impl TryFromStrDomainNetworkAddressExtInfrastructure for NetworkAddress {
    fn try_from_str(s: &str) -> Result<NetworkAddress, AppError> {
        let libp2p_addr = s.parse::<libp2p::Multiaddr>().map_err(|err| {
            AppError::internal(format!(
                "Failed to parse network address as multiaddr: {err}"
            ))
        })?;
        let address = libp2p_addr.try_into_domain_network_address()?;
        Ok(address)
    }
}

pub(crate) trait TaliroNetworkRequestBuilderExtInfrastructure {
    fn build_taliro_request(&self) -> crate::network::protocol::TaliroProtocolRequest;
}

impl TaliroNetworkRequestBuilderExtInfrastructure
    for domain::system::network::event::TaliroNetworkData
{
    fn build_taliro_request(&self) -> crate::network::protocol::TaliroProtocolRequest {
        match self {
            domain::system::network::event::TaliroNetworkData::GetBlockchainTip => {
                crate::network::protocol::TaliroProtocolRequest::GetBlockchainTip
            }
            domain::system::network::event::TaliroNetworkData::GetBlockByHeight(height) => {
                crate::network::protocol::TaliroProtocolRequest::GetBlockByHeight(height.clone())
            }
            domain::system::network::event::TaliroNetworkData::GetBlocksByHeightRange(range) => {
                crate::network::protocol::TaliroProtocolRequest::GetBlocksByHeightRange(
                    range.clone(),
                )
            }
        }
    }
}

pub(crate) trait TryIntoLibp2pPeerIdExtInfrastructure {
    fn try_into_libp2p_peer_id(self) -> Result<libp2p::PeerId, AppError>;
}

impl TryIntoLibp2pPeerIdExtInfrastructure for NetworkPeerId {
    fn try_into_libp2p_peer_id(self) -> Result<libp2p::PeerId, AppError> {
        libp2p::PeerId::from_bytes(self.as_bytes())
            .map_err(|err| AppError::internal(format!("Invalid peer id: {err}")))
    }
}

pub(crate) trait MultiaddrPushPeerIdExtInfrastructure {
    fn set_peer_id(&mut self, peer_id: libp2p::PeerId);
}

impl MultiaddrPushPeerIdExtInfrastructure for libp2p::Multiaddr {
    fn set_peer_id(&mut self, peer_id: libp2p::PeerId) {
        let mut replaced = false;

        let new_self = self
            .clone()
            .into_iter()
            .map(|proto| {
                #[allow(clippy::collapsible_if)]
                if let Protocol::P2p(_) = proto {
                    if !replaced {
                        replaced = true;
                        return Protocol::P2p(peer_id.clone());
                    }
                }
                proto
            })
            .collect();

        if replaced {
            *self = new_self;
        } else {
            self.push(Protocol::P2p(peer_id));
        }
    }
}

pub(crate) trait MultiaddrEphemeralPortExtInfrastructure {
    fn has_ephemeral_port(&self) -> bool;
}

impl MultiaddrEphemeralPortExtInfrastructure for libp2p::Multiaddr {
    fn has_ephemeral_port(&self) -> bool {
        const OS_ASSIGNED_PORT: u16 = 0;

        for protocol in self.iter() {
            #[allow(clippy::collapsible_if)]
            if let Protocol::Tcp(port) = protocol {
                if port == OS_ASSIGNED_PORT {
                    return true;
                }
            }
        }
        false
    }
}
