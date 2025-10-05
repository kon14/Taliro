use super::event::AppNetworkEvent;
use crate::network::protocol::{BlockchainProtocolExchangeCodec, TaliroProtocol};
use common::error::AppError;
use libp2p::request_response::ProtocolSupport;
use libp2p::{gossipsub, identity, kad, request_response, swarm};
use std::iter;

#[derive(swarm::NetworkBehaviour)]
#[behaviour(out_event = "AppNetworkEvent")]
pub(crate) struct AppNetworkBehavior {
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    blockchain: request_response::Behaviour<BlockchainProtocolExchangeCodec>,
}

impl AppNetworkBehavior {
    pub(super) fn new(keys: &identity::Keypair) -> Result<Self, AppError> {
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keys.clone()),
            gossipsub::Config::default(),
        )
        .map_err(|err| AppError::internal(format!("Gossipsub Error: {}", err)))?;
        let local_id = keys.public().to_peer_id();
        let kademlia = kad::Behaviour::new(local_id, kad::store::MemoryStore::new(local_id));
        let blockchain = request_response::Behaviour::new(
            iter::once((TaliroProtocol(), ProtocolSupport::Full)),
            Default::default(),
        );
        let behavior = AppNetworkBehavior {
            gossipsub,
            kademlia,
            blockchain,
        };
        Ok(behavior)
    }

    pub(super) fn get_gossipsub_mut(&mut self) -> &mut gossipsub::Behaviour {
        &mut self.gossipsub
    }

    pub(super) fn get_kademlia_mut(&mut self) -> &mut kad::Behaviour<kad::store::MemoryStore> {
        &mut self.kademlia
    }

    pub(super) fn get_blockchain_mut(
        &mut self,
    ) -> &mut request_response::Behaviour<BlockchainProtocolExchangeCodec> {
        &mut self.blockchain
    }
}
