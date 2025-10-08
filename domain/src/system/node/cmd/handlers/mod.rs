mod blockchain;
mod mempool;
mod network;
mod utxo;

pub(crate) use blockchain::BlockchainCommandHandler;
pub(crate) use mempool::MempoolCommandHandler;
pub(crate) use network::NetworkCommandHandler;
pub(crate) use utxo::UtxoCommandHandler;

use super::{
    BlockchainCommand, MempoolCommand, NetworkCommand, NodeCommandRequest, P2PCommand,
    SystemCommand, UtxoCommand,
};
use common::error::AppError;

#[derive(Debug)]
pub enum CommandHandlerControlFlow {
    Continue,
    Shutdown,
}

/// Main node command dispatcher facade, routing commands to appropriate handlers.
#[derive(Debug)]
pub(crate) struct CommandDispatcher {
    blockchain_handler: BlockchainCommandHandler,
    mempool_handler: MempoolCommandHandler,
    network_handler: NetworkCommandHandler,
    utxo_handler: UtxoCommandHandler,
}

impl CommandDispatcher {
    pub(crate) fn new(
        blockchain_handler: BlockchainCommandHandler,
        mempool_handler: MempoolCommandHandler,
        network_handler: NetworkCommandHandler,
        utxo_handler: UtxoCommandHandler,
    ) -> Self {
        Self {
            blockchain_handler,
            mempool_handler,
            network_handler,
            utxo_handler,
        }
    }

    pub(crate) async fn dispatch(
        &self,
        cmd: NodeCommandRequest,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        match cmd {
            NodeCommandRequest::Blockchain(blockchain_cmd) => {
                self.dispatch_blockchain(blockchain_cmd).await
            }
            NodeCommandRequest::Mempool(mempool_cmd) => self.dispatch_mempool(mempool_cmd).await,
            NodeCommandRequest::Utxo(utxo_cmd) => self.dispatch_utxo(utxo_cmd).await,
            NodeCommandRequest::Network(network_cmd) => self.dispatch_network(network_cmd).await,
            NodeCommandRequest::P2P(p2p_cmd) => self.dispatch_p2p(p2p_cmd).await,
            NodeCommandRequest::System(system_cmd) => self.dispatch_system(system_cmd).await,
        }
    }

    async fn dispatch_blockchain(
        &self,
        cmd: BlockchainCommand,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        match cmd {
            BlockchainCommand::InitiateGenesis(cfg, responder) => {
                self.blockchain_handler
                    .handle_initiate_genesis(cfg, responder)
                    .await
            }
            BlockchainCommand::HandleMineBlock(block_tpl, responder) => {
                self.blockchain_handler
                    .handle_mine_block(block_tpl, responder)
                    .await
            }
            BlockchainCommand::HandleBlockAppend(block, responder) => {
                self.blockchain_handler
                    .handle_block_append(block, responder)
                    .await
            }
            BlockchainCommand::GetTipInfo(responder) => {
                self.blockchain_handler.handle_get_tip_info(responder).await
            }
            BlockchainCommand::GetBlock(block_hash, responder) => {
                self.blockchain_handler
                    .handle_get_block(block_hash, responder)
                    .await
            }
            BlockchainCommand::GetBlockByHeight(block_height, responder) => {
                self.blockchain_handler
                    .handle_get_block_by_height(block_height, responder)
                    .await
            }
            BlockchainCommand::GetBlocksByHeightRange(height_range, responder) => {
                self.blockchain_handler
                    .handle_get_blocks_by_height_range(height_range, responder)
                    .await
            }
        }
    }

    async fn dispatch_mempool(
        &self,
        cmd: MempoolCommand,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        match cmd {
            MempoolCommand::PlaceTransaction(tx, responder) => {
                self.mempool_handler
                    .handle_place_transaction(tx, responder)
                    .await
            }
            MempoolCommand::GetPaginatedTransactions(pagination, responder) => {
                self.mempool_handler
                    .handle_get_paginated_transactions(pagination, responder)
                    .await
            }
            MempoolCommand::GetTransactionsByHashes(tx_hashes, responder) => {
                self.mempool_handler
                    .handle_get_transactions_by_hashes(tx_hashes, responder)
                    .await
            }
        }
    }

    async fn dispatch_utxo(&self, cmd: UtxoCommand) -> Result<CommandHandlerControlFlow, AppError> {
        match cmd {
            UtxoCommand::GetUtxosByOutpoints(outpoints, responder) => {
                self.utxo_handler
                    .handle_get_utxos_by_outpoints(outpoints, responder)
                    .await
            }
            UtxoCommand::GetUtxos(responder) => self.utxo_handler.handle_get_utxos(responder).await,
        }
    }

    async fn dispatch_network(
        &self,
        cmd: NetworkCommand,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        match cmd {
            NetworkCommand::GetSelfInfo(responder) => {
                self.network_handler.handle_get_self_info(responder).await
            }
            NetworkCommand::GetPeers(responder) => {
                self.network_handler.handle_get_peers(responder).await
            }
            NetworkCommand::AddPeer(network_address, responder) => {
                self.network_handler
                    .handle_add_peer(network_address, responder)
                    .await
            }
        }
    }

    async fn dispatch_p2p(&self, cmd: P2PCommand) -> Result<CommandHandlerControlFlow, AppError> {
        match cmd {
            P2PCommand::HandleReceiveBlockchainTipInfo(origin_peer_id, block_info, responder) => {
                self.network_handler
                    .handle_receive_blockchain_tip_info(origin_peer_id, block_info, responder)
                    .await
            }
            P2PCommand::HandleReceiveBlocks(origin_peer_id, blocks, responder) => {
                self.network_handler
                    .handle_receive_blocks(origin_peer_id, blocks, responder)
                    .await
            }
            P2PCommand::ProxyForwardNetworkEvent(event, responder) => {
                self.network_handler
                    .handle_forward_network_event(event, responder)
                    .await
            }
        }
    }

    async fn dispatch_system(
        &self,
        cmd: SystemCommand,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        match cmd {
            SystemCommand::RequestNodeShutdown => Ok(CommandHandlerControlFlow::Shutdown),
        }
    }
}
