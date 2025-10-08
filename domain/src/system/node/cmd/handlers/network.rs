use super::super::CommandResponder;
use super::CommandHandlerControlFlow;
use crate::entities::block::{BlockHeight, NonValidatedBlock};
use crate::system::blockchain::Blockchain;
use crate::system::network::P2PNetworkHandle;
use crate::system::network::event::{AddPeerResponse, NetworkEvent};
use crate::system::queue::BlockSyncQueue;
use crate::types::hash::Hash;
use crate::types::network::{NetworkAddress, NetworkIdentityKeypair, NetworkPeerId};
use common::error::{AppError, NetworkError};
use common::{log_node_debug, log_node_error};
use std::sync::Arc;

/// Handles network/P2P-related commands.
#[derive(Debug, Clone)]
pub(crate) struct NetworkCommandHandler {
    network: Arc<dyn P2PNetworkHandle>,
    blockchain: Arc<dyn Blockchain>,
    block_sync_queue: Arc<dyn BlockSyncQueue>,
}

impl NetworkCommandHandler {
    pub(crate) fn new(
        network: Arc<dyn P2PNetworkHandle>,
        blockchain: Arc<dyn Blockchain>,
        block_sync_queue: Arc<dyn BlockSyncQueue>,
    ) -> Self {
        Self {
            network,
            blockchain,
            block_sync_queue,
        }
    }

    /// Handle receiving blockchain tip info from a peer.
    pub(in crate::system::node) async fn handle_receive_blockchain_tip_info(
        &self,
        origin_peer_id: NetworkPeerId,
        block_info: Option<(Hash, BlockHeight)>,
        responder: Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "NetworkCommandHandler: Received blockchain tip info from peer: {}",
            origin_peer_id
        );

        let res = self
            .handle_receive_blockchain_tip_info_internal(origin_peer_id, block_info)
            .await;

        if let Err(ref err) = res {
            log_node_error!("Failed to handle blockchain tip info: {}", err);
        }

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    async fn handle_receive_blockchain_tip_info_internal(
        &self,
        origin_peer_id: NetworkPeerId,
        block_info: Option<(Hash, BlockHeight)>,
    ) -> Result<(), AppError> {
        let Some(tip_info) = block_info else {
            return Ok(());
        };

        let Some(unknown_heights) = self.blockchain.get_unknown_block_heights(tip_info).await?
        else {
            return Ok(());
        };

        log_node_debug!(
            "Requesting unknown blocks in range: {:?} from peer: {}",
            unknown_heights,
            origin_peer_id
        );

        for height in unknown_heights.start().as_u64()..=unknown_heights.end().as_u64() {
            let height = height.into();

            if self.block_sync_queue.is_in_progress(&height).await {
                continue;
            }

            self.block_sync_queue
                .request_block(height.clone(), origin_peer_id.clone())
                .await
                .map_err(|err| {
                    AppError::internal(format!(
                        "Failed to request block from peer! Height: {} | Peer: {}, Error: {}",
                        height, origin_peer_id, err,
                    ))
                })?;
        }
        Ok(())
    }

    /// Handle receiving blocks from a peer.
    pub(in crate::system::node) async fn handle_receive_blocks(
        &self,
        origin_peer_id: NetworkPeerId,
        blocks: Vec<NonValidatedBlock>,
        responder: Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "NetworkCommandHandler: Received {} block(s) from peer: {}",
            blocks.len(),
            origin_peer_id
        );

        let res = self
            .handle_receive_blocks_internal(origin_peer_id, blocks)
            .await;

        if let Err(ref err) = res {
            log_node_error!("Failed to handle received blocks: {}", err);
        }

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    async fn handle_receive_blocks_internal(
        &self,
        origin_peer_id: NetworkPeerId,
        blocks: Vec<NonValidatedBlock>,
    ) -> Result<(), AppError> {
        for block in blocks {
            let block_hash = block.get_hash();

            // Skip if we already have this block.
            if self.blockchain.has_canon_block(&block_hash).await? {
                log_node_debug!("Skipping already known block: {}", block_hash);
                continue;
            }

            self.block_sync_queue
                .on_block_received(block, origin_peer_id.clone())
                .await;
        }
        Ok(())
    }

    /// Get self network information.
    pub(in crate::system::node) async fn handle_get_self_info(
        &self,
        responder: Box<
            dyn CommandResponder<Result<(NetworkIdentityKeypair, Vec<NetworkAddress>), AppError>>
                + Send,
        >,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("NetworkCommandHandler: Getting self info");

        let res = self.get_self_info_internal().await;

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    async fn get_self_info_internal(
        &self,
    ) -> Result<(NetworkIdentityKeypair, Vec<NetworkAddress>), AppError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let network_event = NetworkEvent::GetSelfInfo(tx);

        self.network.publish_network_event(network_event)?;

        rx.await.map_err(|err| {
            AppError::internal(format!("Failed to retrieve network info! | Error: {err}"))
        })
    }

    /// Get connected peers.
    pub(in crate::system::node) async fn handle_get_peers(
        &self,
        responder: Box<dyn CommandResponder<Result<Vec<NetworkAddress>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("NetworkCommandHandler: Getting peers");

        let res = self.get_peers_internal().await;

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    async fn get_peers_internal(&self) -> Result<Vec<NetworkAddress>, AppError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let network_event = NetworkEvent::GetPeers(tx);

        self.network.publish_network_event(network_event)?;

        rx.await.map_err(|err| {
            AppError::internal(format!("Failed to retrieve network peers! | Error: {err}"))
        })
    }

    /// Add a new peer to the network.
    pub(in crate::system::node) async fn handle_add_peer(
        &self,
        network_address: NetworkAddress,
        responder: Box<dyn CommandResponder<Result<AddPeerResponse, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("NetworkCommandHandler: Adding peer: {}", network_address);

        let res = self.add_peer_internal(network_address).await;

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    async fn add_peer_internal(
        &self,
        network_address: NetworkAddress,
    ) -> Result<AddPeerResponse, AppError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let network_event = NetworkEvent::AddPeer(network_address, tx);

        self.network.publish_network_event(network_event)?;

        rx.await.map_err(|err| {
            AppError::Network(NetworkError::PeerConnectionFailed {
                reason: err.to_string(),
            })
        })
    }

    /// Forward a network event (proxy).
    pub(in crate::system::node) async fn handle_forward_network_event(
        &self,
        event: NetworkEvent,
        responder: Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("NetworkCommandHandler: Forwarding network event");

        let res = self.network.publish_network_event(event);

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }
}
