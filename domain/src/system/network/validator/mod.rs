use crate::types::network::{NetworkAddress, NetworkPeerId};
use common::error::AppError;
use std::fmt::Debug;

/// An abstraction layer over network data type validations.<br />
/// Allows for infrastructure-agnostic handling of network data types.
#[cfg_attr(test, mockall::automock)]
pub trait NetworkEntityValidator: Send + Sync + Debug {
    /// Validates an address string, returning a [`NetworkAddress`].<br />
    /// Expects a valid multiaddr with a specified P2P peer ID!
    fn validate_address(&self, multiaddr: String) -> Result<NetworkAddress, AppError>;

    /// Validates a peer ID string, returning a [`NetworkPeerId`].
    fn validate_peer_id(&self, peer_id: String) -> Result<NetworkPeerId, AppError>;
}
