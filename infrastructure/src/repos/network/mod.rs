use crate::ext::AppErrorExtInfrastructure;
use crate::storage::SledStorage;
use common::error::AppError;
use domain::encode::{TryDecode, TryEncode};
use domain::repos::network::NetworkRepository;
use domain::types::network::{NetworkAddress, NetworkIdentityKeypair};
use sled::Tree;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

pub struct SledNetworkRepository {
    peer_address_tree: Tree,
    meta_tree: Tree,
}

impl Debug for SledNetworkRepository {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SledBlockchainRepository")
            .field("peer_address_tree", &SledStorage::NETWORK_PEER_ADDRESS_TREE)
            .finish()
    }
}

impl SledNetworkRepository {
    pub fn open(peer_address_tree: Tree, meta_tree: Tree) -> Result<Self, AppError> {
        let repo = Self {
            peer_address_tree,
            meta_tree,
        };
        Ok(repo)
    }
}

impl NetworkRepository for SledNetworkRepository {
    fn insert_peer_address(&self, address: NetworkAddress) -> Result<(), AppError> {
        let data = address.try_encode()?;
        self.peer_address_tree
            .insert(address.get_address_bytes(), data)
            .to_app_error()?;
        Ok(())
    }

    fn get_peer_addresses(&self) -> Result<HashSet<NetworkAddress>, AppError> {
        let mut peer_addresses = Vec::new();
        for addr in self.peer_address_tree.iter() {
            let (_key, value) = addr.to_app_error()?;
            let address = NetworkAddress::try_decode(&value)?;
            peer_addresses.push(address);
        }
        let peer_addresses_set: HashSet<_> = peer_addresses.into_iter().collect();
        Ok(peer_addresses_set)
    }

    fn delete_peer_address(&self, address: &NetworkAddress) -> Result<(), AppError> {
        self.peer_address_tree
            .remove(address.get_address_bytes())
            .to_app_error()?;
        Ok(())
    }

    fn insert_identity_keys(&self, keys: NetworkIdentityKeypair) -> Result<(), AppError> {
        let data = keys.try_encode()?;
        self.meta_tree
            .insert(SledStorage::NETWORK_META_TREE_IDENTITY_KEY_PAIR_KEY, data)
            .to_app_error()?;
        Ok(())
    }

    fn get_identity_keys(&self) -> Result<Option<NetworkIdentityKeypair>, AppError> {
        let key_pair = self
            .meta_tree
            .get(SledStorage::NETWORK_META_TREE_IDENTITY_KEY_PAIR_KEY)
            .to_app_error()?;
        key_pair
            .map(|pair| NetworkIdentityKeypair::try_decode(&pair))
            .transpose()
    }
}
