#[cfg(test)]
mod tests;

use common::error::AppError;
use domain::system::network::validator::NetworkEntityValidator;
use domain::types::network::{NetworkAddress, NetworkPeerId};
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};

#[derive(Debug)]
pub struct Libp2pNetworkEntityValidator;

impl NetworkEntityValidator for Libp2pNetworkEntityValidator {
    fn validate_address(&self, multiaddr: String) -> Result<NetworkAddress, AppError> {
        if multiaddr.trim().is_empty() {
            return Err(AppError::internal(
                "Invalid multiaddr format! Multiaddr cannot be empty".to_string(),
            ));
        }

        let multiaddr = multiaddr.parse::<Multiaddr>().map_err(|err| {
            AppError::internal(format!(
                "Invalid multiaddr ({multiaddr}) format! | Parse Error: {err}"
            ))
        })?;

        // Verify P2P Peer ID presence
        let peer_id = multiaddr.iter().find_map(|proto| match proto {
            Protocol::P2p(peer_id) => Some(peer_id),
            _ => None,
        });
        let Some(peer_id) = peer_id else {
            return Err(AppError::internal(format!(
                "Invalid multiaddr format! Multiaddr ({multiaddr}) doesn't contain a valid P2P peer ID!"
            )));
        };
        let network_peer_id =
            NetworkPeerId::_new_validated(peer_id.to_bytes(), peer_id.to_string());

        let address = NetworkAddress::_new_validated(
            multiaddr.to_string().into_bytes(),
            multiaddr.to_string(),
            network_peer_id,
        );
        Ok(address)
    }

    fn validate_peer_id(&self, peer_id: String) -> Result<NetworkPeerId, AppError> {
        // Reject empty strings upfront
        if peer_id.trim().is_empty() {
            return Err(AppError::internal(
                "Invalid peer_id format! Peer ID cannot be empty".to_string(),
            ));
        }

        let peer_id = peer_id.parse::<PeerId>().map_err(|err| {
            AppError::internal(format!(
                "Invalid peer_id ({peer_id}) format! | Parse Error: {err}"
            ))
        })?;
        let peer_id = NetworkPeerId::_new_validated(peer_id.to_bytes(), peer_id.to_string());
        Ok(peer_id)
    }
}
