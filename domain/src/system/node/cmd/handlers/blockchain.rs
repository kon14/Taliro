use super::super::CommandResponder;
use super::CommandHandlerControlFlow;
use crate::entities::block::{Block, BlockHeight, BlockTemplate, NonValidatedBlock};
use crate::genesis::config::GenesisConfig;
use crate::system::blockchain::Blockchain;
use crate::system::mempool::Mempool;
use crate::system::network::P2PNetworkHandle;
use crate::system::network::event::{GossipsubNetworkEvent, NetworkEvent};
use crate::system::utxo::UtxoSetWriter;
use crate::system::validation::block::BlockValidator;
use crate::types::hash::Hash;
use common::error::AppError;
use common::{log_node_debug, log_node_error};
use std::ops::RangeInclusive;
use std::sync::Arc;

/// Handles blockchain-related commands.
#[derive(Debug, Clone)]
pub(crate) struct BlockchainCommandHandler {
    blockchain: Arc<dyn Blockchain>,
    block_validator: Arc<dyn BlockValidator>,
    utxo_set_w: Arc<dyn UtxoSetWriter>,
    mempool: Arc<dyn Mempool>,
    network: Arc<dyn P2PNetworkHandle>,
}

impl BlockchainCommandHandler {
    pub(crate) fn new(
        blockchain: Arc<dyn Blockchain>,
        block_validator: Arc<dyn BlockValidator>,
        utxo_set_w: Arc<dyn UtxoSetWriter>,
        mempool: Arc<dyn Mempool>,
        network: Arc<dyn P2PNetworkHandle>,
    ) -> Self {
        Self {
            blockchain,
            block_validator,
            utxo_set_w,
            mempool,
            network,
        }
    }

    /// Handle genesis block creation.
    pub(in crate::system::node) async fn handle_initiate_genesis(
        &self,
        cfg: GenesisConfig,
        responder: Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("BlockchainCommandHandler: Initiating genesis");

        let res = async {
            let block = NonValidatedBlock::new_genesis(cfg)?;
            let block = self.block_validator.validate_block(block).await?;
            self.blockchain.add_block(block).await?;
            Ok(())
        }
        .await;

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Handle mining a new block.
    pub(in crate::system::node) async fn handle_mine_block(
        &self,
        block_tpl: BlockTemplate,
        responder: Box<dyn CommandResponder<Result<Block, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("BlockchainCommandHandler: Mining block");

        let res = async {
            let block = NonValidatedBlock::from_template(block_tpl)?;
            let block = self.block_validator.validate_block(block).await?;
            self.blockchain.add_block(block.clone()).await?;
            Ok(block)
        }
        .await;

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Handle block append operation.
    pub(in crate::system::node) async fn handle_block_append(
        &self,
        block: Block,
        responder: Box<dyn CommandResponder<Result<(), AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        // TODO: Implement idempotency checks.
        // TODO: Implement proper transaction boundaries and rollback mechanism.

        log_node_debug!(
            "BlockchainCommandHandler: Appending block | Height: {} | Hash: {}",
            block.get_height(),
            block.get_hash()
        );

        let res = self.append_block_internal(block).await;

        if let Err(ref err) = res {
            log_node_error!("Failed to append block! | Error: {}", err);
        }

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    async fn append_block_internal(&self, block: Block) -> Result<(), AppError> {
        // Apply to UTXO set
        self.utxo_set_w.apply_block(&block).map_err(|err| {
            AppError::internal(format!(
                "Failed to apply block to UTXO set! | Hash: {} | Height: {} | Error: {}",
                block.get_hash(),
                block.get_height(),
                err,
            ))
        })?;

        // Apply to Mempool
        self.mempool.apply_block(&block).await.map_err(|err| {
            log_node_error!(
                "UTXO set updated but mempool update failed! Manual intervention may be required."
            );
            AppError::internal(format!(
                "Failed to apply block to mempool! | Block height: {}, Hash: {}, Error: {}",
                block.get_height(),
                block.get_hash(),
                err
            ))
        })?;

        // Update Blockchain tip
        self.blockchain.set_tip(&block).await.map_err(|err| {
            log_node_error!(
                "UTXO and mempool updated but blockchain tip update failed! System state inconsistent."
            );
            AppError::internal(
                format!(
                    "Failed to update blockchain tip! | Block height: {}, Hash: {}, Error: {}",
                    block.get_height(),
                    block.get_hash(),
                    err
                ),
            )
        })?;

        // Broadcast to Network
        let network_event =
            NetworkEvent::Gossipsub(GossipsubNetworkEvent::BroadcastNewBlock(block.clone()));

        if let Err(err) = self.network.publish_network_event(network_event) {
            log_node_error!(
                "Block appended successfully but network broadcast failed: {} | Block height: {}, Hash: {}",
                err,
                block.get_height(),
                block.get_hash()
            );
            // Don't fail the entire operation if broadcast fails...
        }

        Ok(())
    }

    /// Get blockchain tip information.
    pub(in crate::system::node) async fn handle_get_tip_info(
        &self,
        responder: Box<dyn CommandResponder<Result<Option<(Hash, BlockHeight)>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("BlockchainCommandHandler: Getting tip info");

        let res = self.blockchain.get_tip_info().await;
        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Get block by hash.
    pub(in crate::system::node) async fn handle_get_block(
        &self,
        block_hash: Hash,
        responder: Box<dyn CommandResponder<Result<Option<Block>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "BlockchainCommandHandler: Getting block by hash: {}",
            block_hash
        );

        let res = self.blockchain.get_canon_block(&block_hash).await;
        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Get block by height.
    pub(in crate::system::node) async fn handle_get_block_by_height(
        &self,
        block_height: BlockHeight,
        responder: Box<dyn CommandResponder<Result<Option<Block>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "BlockchainCommandHandler: Getting block by height: {}",
            block_height
        );

        let res = self
            .blockchain
            .get_canon_block_by_height(&block_height)
            .await;
        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Get blocks by height range.
    pub(in crate::system::node) async fn handle_get_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
        responder: Box<dyn CommandResponder<Result<Vec<Block>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "BlockchainCommandHandler: Getting blocks by height range: {:?}",
            height_range
        );

        let res = self
            .blockchain
            .get_canon_blocks_by_height_range(height_range)
            .await;
        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }
}
