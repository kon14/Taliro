pub mod handlers;

use crate::entities::block::{Block, BlockHeight, BlockTemplate, NonValidatedBlock};
use crate::entities::transaction::{
    NonValidatedTransaction, Transaction, TransactionOutPoint, Utxo,
};
use crate::genesis::config::GenesisConfig;
use crate::system::network::event::{AddPeerResponse, NetworkEvent};
use crate::types::hash::Hash;
use crate::types::network::{NetworkAddress, NetworkIdentityKeypair, NetworkPeerId};
use async_trait::async_trait;
use common::error::AppError;
use common::params::PaginationParams;
use derivative::Derivative;
use std::fmt::Debug;
use std::future::Future;
use std::ops::RangeInclusive;
use std::pin::Pin;

// ============================================================================
// Domain-Specific Command Enums
// ============================================================================

#[derive(Derivative)]
#[derivative(Debug)]
pub enum BlockchainCommand {
    /// Dev-administered command to initiate genesis process.
    InitiateGenesis(
        GenesisConfig,
        #[derivative(Debug = "ignore")] Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ),
    /// Handles mining a new block.
    HandleMineBlock(
        BlockTemplate,
        #[derivative(Debug = "ignore")] Box<dyn CommandResponder<Result<Block, AppError>> + Send>,
    ),
    /// Post-blockchain insertion command to handle updating subsystems and incrementing active height.
    HandleBlockAppend(
        Block,
        #[derivative(Debug = "ignore")] Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ),
    /// Dev-administered command to retrieve blockchain tip information.
    GetTipInfo(
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Option<(Hash, BlockHeight)>, AppError>> + Send>,
    ),
    /// Dev-administered command to retrieve blockchain block.
    GetBlock(
        Hash,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Option<Block>, AppError>> + Send>,
    ),
    /// Dev-administered command to retrieve blockchain block by height.
    GetBlockByHeight(
        BlockHeight,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Option<Block>, AppError>> + Send>,
    ),
    /// Dev-administered command to retrieve blockchain blocks by height range (inclusive).
    GetBlocksByHeightRange(
        RangeInclusive<BlockHeight>,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Vec<Block>, AppError>> + Send>,
    ),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum MempoolCommand {
    /// Dev-administered command to place a transaction into the mempool.
    PlaceTransaction(
        NonValidatedTransaction,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Transaction, AppError>> + Send>,
    ),
    /// Dev-administered command to retrieve mempool transactions with pagination.<br />
    /// Returns the paginated transactions along with the total transaction count.
    GetPaginatedTransactions(
        PaginationParams,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<(Vec<Transaction>, usize), AppError>> + Send>,
    ),
    /// Dev-administered command to retrieve mempool transactions by their hashes.
    GetTransactionsByHashes(
        Vec<Hash>,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Vec<Transaction>, AppError>> + Send>,
    ),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum UtxoCommand {
    /// Dev-administered command to retrieve UTXOs by their outpoints.
    GetUtxosByOutpoints(
        Vec<TransactionOutPoint>,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Vec<Utxo>, AppError>> + Send>,
    ),
    /// Dev-administered command to retrieve all UTXOs.
    GetUtxos(
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Vec<Utxo>, AppError>> + Send>,
    ),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum NetworkCommand {
    /// Dev-administered command to retrieve this node's network information.
    GetSelfInfo(
        #[derivative(Debug = "ignore")]
        Box<
            dyn CommandResponder<Result<(NetworkIdentityKeypair, Vec<NetworkAddress>), AppError>>
                + Send,
        >,
    ),
    /// Dev-administered command to retrieve this network's connected peers.
    GetPeers(
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<Vec<NetworkAddress>, AppError>> + Send>,
    ),
    /// Dev-administered command to connect the network to a new peer.
    AddPeer(
        NetworkAddress,
        #[derivative(Debug = "ignore")]
        Box<dyn CommandResponder<Result<AddPeerResponse, AppError>> + Send>,
    ),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum P2PCommand {
    /// Handles receiving blockchain tip info from a peer.
    HandleReceiveBlockchainTipInfo(
        NetworkPeerId,
        Option<(Hash, BlockHeight)>,
        #[derivative(Debug = "ignore")] Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ),
    /// Handles receiving blocks from a peer.
    HandleReceiveBlocks(
        NetworkPeerId,
        Vec<NonValidatedBlock>,
        #[derivative(Debug = "ignore")] Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ),
    /// Forwards a network event to the appropriate subsystem handler.<br />
    /// Used to decouple subsystems dependent on network event publishing from `P2PNetworkHandle`.
    ProxyForwardNetworkEvent(
        NetworkEvent,
        #[derivative(Debug = "ignore")] Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum SystemCommand {
    /// Internal command to initiate graceful node termination.
    RequestNodeShutdown,
}

// ============================================================================
// Top-Level Command Enum
// ============================================================================

#[derive(Derivative)]
#[derivative(Debug)]
pub enum NodeCommandRequest {
    Blockchain(BlockchainCommand),
    Mempool(MempoolCommand),
    Utxo(UtxoCommand),
    Network(NetworkCommand),
    P2P(P2PCommand),
    System(SystemCommand),
}

// ============================================================================
// Traits
// ============================================================================

#[async_trait]
pub trait CommandSender: Send + Sync + Debug {
    async fn send(&self, cmd: NodeCommandRequest) -> Result<(), AppError>;
}

#[async_trait]
pub trait CommandReceiver: Send {
    async fn receive(&mut self) -> Option<NodeCommandRequest>;
}

pub trait CommandResponder<T>: Send + Debug {
    fn respond(self: Box<Self>, value: T);
}

// ============================================================================
// Command Factory Traits
// ============================================================================

/// A factory for building cmd command requests along with their associated response futures.<br />
/// Published events are consumed in [`crate::system::node::state::run::NodeRunning`].
pub trait CommandResponderFactory: Send + Sync + Debug {
    // Blockchain commands
    fn build_blk_cmd_init_genesis(
        &self,
        cfg: GenesisConfig,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    );

    fn build_blk_cmd_handle_mine_block(
        &self,
        block_tpl: BlockTemplate,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Block, AppError>> + Send>>,
    );

    fn build_blk_cmd_handle_block_append(
        &self,
        block: Block,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    );

    fn build_blk_cmd_get_tip_info(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<(Hash, BlockHeight)>, AppError>> + Send>>,
    );

    fn build_blk_cmd_get_block(
        &self,
        block_hash: Hash,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<Block>, AppError>> + Send>>,
    );

    fn build_blk_cmd_get_block_by_height(
        &self,
        height: BlockHeight,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<Block>, AppError>> + Send>>,
    );

    fn build_blk_cmd_get_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Block>, AppError>> + Send>>,
    );

    // Mempool commands
    fn build_mp_cmd_place_transaction(
        &self,
        tx: NonValidatedTransaction,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send>>,
    );

    fn build_mp_get_paginated_transactions(
        &self,
        pagination: PaginationParams,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(Vec<Transaction>, usize), AppError>> + Send>>,
    );

    fn build_mp_cmd_get_transactions_by_hashes(
        &self,
        tx_hashes: Vec<Hash>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send>>,
    );

    // UTXO commands
    fn build_utxo_get_utxos_by_outpoints(
        &self,
        outpoints: Vec<TransactionOutPoint>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Utxo>, AppError>> + Send>>,
    );

    fn build_utxo_cmd_get_utxos(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Utxo>, AppError>> + Send>>,
    );

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
    );

    fn build_net_cmd_get_peers(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<NetworkAddress>, AppError>> + Send>>,
    );

    fn build_net_cmd_add_peer(
        &self,
        network_address: NetworkAddress,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<AddPeerResponse, AppError>> + Send>>,
    );

    // P2P protocol commands
    fn build_p2p_cmd_receive_blockchain_tip_info(
        &self,
        origin_peer_id: NetworkPeerId,
        block_info: Option<(Hash, BlockHeight)>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    );

    fn build_p2p_cmd_receive_blocks(
        &self,
        origin_peer_id: NetworkPeerId,
        blocks: Vec<NonValidatedBlock>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    );

    fn build_proxy_cmd_forward_network_event(
        &self,
        event: NetworkEvent,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    );
}
