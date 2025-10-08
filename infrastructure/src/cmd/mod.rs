use async_trait::async_trait;
use common::error::AppError;
use common::params::PaginationParams;
use domain::entities::block::{Block, BlockHeight, BlockTemplate, NonValidatedBlock};
use domain::entities::transaction::{
    NonValidatedTransaction, Transaction, TransactionOutPoint, Utxo,
};
use domain::genesis::config::GenesisConfig;
use domain::system::network::event::{AddPeerResponse, NetworkEvent};
use domain::system::node::cmd::{
    BlockchainCommand, CommandReceiver, CommandResponder, CommandResponderFactory, CommandSender,
    MempoolCommand, NetworkCommand, NodeCommandRequest, P2PCommand, UtxoCommand,
};
use domain::types::hash::Hash;
use domain::types::network::{NetworkAddress, NetworkIdentityKeypair, NetworkPeerId};
use std::fmt::Debug;
use std::future::Future;
use std::ops::RangeInclusive;
use std::pin::Pin;
use tokio::sync::{mpsc, oneshot};

// ============================================================================
// Command Channel Implementation
// ============================================================================
//
// Uses MPSC channel with Arc-wrapped trait object for shared command sending.
//
// Architecture:
// - Multiple subsystems share one Arc<dyn CommandSender>.
// - Each subsystem can send commands concurrently via &self.
// - Single consumer (NodeRunning event loop) receives all commands.
// - Request-response pattern via oneshot channels for replies.
//
// Rationale:
// - mpsc::Sender chosen for semantic clarity (multiple logical producers).
// - Arc<dyn CommandSender> enables trait object polymorphism across layers.
// - The mpsc::Sender is already internally Arc-based,
//   but the outer Arc allows us to use trait objects (dyn CommandSender).
// ============================================================================

#[derive(Clone, Debug)]
pub struct NodeCommandSender(mpsc::Sender<NodeCommandRequest>);

pub struct NodeCommandReceiver(mpsc::Receiver<NodeCommandRequest>);

pub fn build_channel(buffer_size: usize) -> (NodeCommandSender, NodeCommandReceiver) {
    let (cmd_tx, cmd_rx) = mpsc::channel(buffer_size);
    let sender = NodeCommandSender(cmd_tx);
    let receiver = NodeCommandReceiver(cmd_rx);
    (sender, receiver)
}

#[async_trait]
impl CommandSender for NodeCommandSender {
    async fn send(&self, cmd: NodeCommandRequest) -> Result<(), AppError> {
        self.0.send(cmd).await.map_err(|err| {
            AppError::internal(format!("Failed to send command event! Error: {}", err))
        })
    }
}

#[async_trait]
impl CommandReceiver for NodeCommandReceiver {
    async fn receive(&mut self) -> Option<NodeCommandRequest> {
        self.0.recv().await
    }
}

// ============================================================================
// Responder Implementation
// ============================================================================

#[derive(Debug)]
pub struct TokioResponder<T: Debug>(oneshot::Sender<T>);

impl<T: Send + Debug> CommandResponder<T> for TokioResponder<T> {
    fn respond(self: Box<Self>, value: T) {
        let _ = self.0.send(value);
    }
}

fn create_command<T, F>(wrapper: F) -> (NodeCommandRequest, Pin<Box<dyn Future<Output = T> + Send>>)
where
    T: Send + Debug + 'static,
    F: FnOnce(Box<dyn CommandResponder<T> + Send>) -> NodeCommandRequest,
{
    let (tx, rx) = oneshot::channel();
    let responder = Box::new(TokioResponder(tx)) as Box<dyn CommandResponder<T> + Send>;
    let command = wrapper(responder);
    let fut = Box::pin(async move { rx.await.expect("Responder dropped without responding!") });
    (command, fut)
}

// ============================================================================
// Factory Implementation
// ============================================================================

#[derive(Debug)]
pub struct NodeCommandResponderFactory;

impl CommandResponderFactory for NodeCommandResponderFactory {
    // Blockchain commands
    fn build_blk_cmd_init_genesis(
        &self,
        cfg: GenesisConfig,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Blockchain(BlockchainCommand::InitiateGenesis(cfg, responder))
        })
    }

    fn build_blk_cmd_handle_mine_block(
        &self,
        block_tpl: BlockTemplate,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Block, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Blockchain(BlockchainCommand::HandleMineBlock(block_tpl, responder))
        })
    }

    fn build_blk_cmd_handle_block_append(
        &self,
        block: Block,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Blockchain(BlockchainCommand::HandleBlockAppend(block, responder))
        })
    }

    fn build_blk_cmd_get_tip_info(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<(Hash, BlockHeight)>, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Blockchain(BlockchainCommand::GetTipInfo(responder))
        })
    }

    fn build_blk_cmd_get_block(
        &self,
        block_hash: Hash,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<Block>, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Blockchain(BlockchainCommand::GetBlock(block_hash, responder))
        })
    }

    fn build_blk_cmd_get_block_by_height(
        &self,
        height: BlockHeight,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<Block>, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Blockchain(BlockchainCommand::GetBlockByHeight(height, responder))
        })
    }

    fn build_blk_cmd_get_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Block>, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Blockchain(BlockchainCommand::GetBlocksByHeightRange(
                height_range,
                responder,
            ))
        })
    }

    // Mempool commands
    fn build_mp_cmd_place_transaction(
        &self,
        transaction: NonValidatedTransaction,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Mempool(MempoolCommand::PlaceTransaction(transaction, responder))
        })
    }

    fn build_mp_get_paginated_transactions(
        &self,
        pagination: PaginationParams,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(Vec<Transaction>, usize), AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Mempool(MempoolCommand::GetPaginatedTransactions(
                pagination, responder,
            ))
        })
    }

    fn build_mp_cmd_get_transactions_by_hashes(
        &self,
        tx_hashes: Vec<Hash>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Mempool(MempoolCommand::GetTransactionsByHashes(
                tx_hashes, responder,
            ))
        })
    }

    // UTXO commands
    fn build_utxo_get_utxos_by_outpoints(
        &self,
        outpoints: Vec<TransactionOutPoint>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Utxo>, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Utxo(UtxoCommand::GetUtxosByOutpoints(outpoints, responder))
        })
    }

    fn build_utxo_cmd_get_utxos(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Utxo>, AppError>> + Send>>,
    ) {
        create_command(|responder| NodeCommandRequest::Utxo(UtxoCommand::GetUtxos(responder)))
    }

    // Network commands
    fn build_net_cmd_get_self_info(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<
            Box<
                dyn Future<Output = Result<(NetworkIdentityKeypair, Vec<NetworkAddress>), AppError>>
                    + Send,
            >,
        >,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Network(NetworkCommand::GetSelfInfo(responder))
        })
    }

    fn build_net_cmd_get_peers(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<NetworkAddress>, AppError>> + Send>>,
    ) {
        create_command(|responder| NodeCommandRequest::Network(NetworkCommand::GetPeers(responder)))
    }

    fn build_net_cmd_add_peer(
        &self,
        network_address: NetworkAddress,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<AddPeerResponse, AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::Network(NetworkCommand::AddPeer(network_address, responder))
        })
    }

    // P2P protocol commands
    fn build_p2p_cmd_receive_blockchain_tip_info(
        &self,
        origin_peer_id: NetworkPeerId,
        block_info: Option<(Hash, BlockHeight)>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::P2P(P2PCommand::HandleReceiveBlockchainTipInfo(
                origin_peer_id,
                block_info,
                responder,
            ))
        })
    }

    fn build_p2p_cmd_receive_blocks(
        &self,
        origin_peer_id: NetworkPeerId,
        blocks: Vec<NonValidatedBlock>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::P2P(P2PCommand::HandleReceiveBlocks(
                origin_peer_id,
                blocks,
                responder,
            ))
        })
    }

    fn build_proxy_cmd_forward_network_event(
        &self,
        event: NetworkEvent,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        create_command(|responder| {
            NodeCommandRequest::P2P(P2PCommand::ProxyForwardNetworkEvent(event, responder))
        })
    }
}
