use crate::types::network::{NetworkAddress, NetworkIdentityKeypair};
use common::error::AppError;
use std::collections::HashSet;
use std::fmt::Debug;

pub trait NetworkRepository: Send + Sync + Debug {
    fn insert_peer_address(&self, address: NetworkAddress) -> Result<(), AppError>;

    fn get_peer_addresses(&self) -> Result<HashSet<NetworkAddress>, AppError>;

    fn delete_peer_address(&self, address: &NetworkAddress) -> Result<(), AppError>;

    fn insert_identity_keys(&self, keys: NetworkIdentityKeypair) -> Result<(), AppError>;

    fn get_identity_keys(&self) -> Result<Option<NetworkIdentityKeypair>, AppError>;
}
