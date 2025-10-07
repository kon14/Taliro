mod identity;
mod peer;

pub use identity::NetworkIdentityKeypair;
pub use peer::NetworkPeerId;
use std::fmt;

use crate::encode::{TryDecode, TryEncode};
use crate::ext::AppErrorConvertibleDomain;
use bincode::{Decode, Encode};
use common::error::AppError;

/// An opaque wrapper over a multiaddr byte representation.
#[derive(Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct NetworkAddress {
    address_bytes: Vec<u8>,
    address_str: String, // includes P2P peer id
    peer_id: NetworkPeerId,
}

impl NetworkAddress {
    pub fn _new_validated(
        address_bytes: Vec<u8>,
        address_str_repr: String,
        peer_id: NetworkPeerId,
    ) -> Self {
        Self {
            address_bytes,
            address_str: address_str_repr,
            peer_id,
        }
    }

    pub fn get_address_bytes(&self) -> Vec<u8> {
        self.address_bytes.clone()
    }

    pub fn get_address_str(&self) -> String {
        self.address_str.clone()
    }

    pub fn get_peer_id(&self) -> NetworkPeerId {
        self.peer_id.clone()
    }
}

impl TryEncode for NetworkAddress {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for NetworkAddress {
    fn try_decode(data: &[u8]) -> Result<Self, AppError> {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}

impl fmt::Display for NetworkAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Address string already includes peer_id suffix if any.
        write!(f, "{}", self.address_str)
    }
}

impl fmt::Debug for NetworkAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetworkAddress({})", self)
    }
}
