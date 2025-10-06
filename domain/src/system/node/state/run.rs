use crate::entities::block::NonValidatedBlock;
use crate::system::blockchain::Blockchain;
use crate::system::mempool::Mempool;
use crate::system::network::P2PNetworkHandle;
use crate::system::network::event::{GossipsubNetworkEvent, NetworkEvent};
use crate::system::node::bus::{CommandReceiver, NodeCommandRequest};
use crate::system::node::state::exit::NodeTerminating;
use crate::system::node::state::start::NodeStarted;
use crate::system::queue::{BlockProcessingQueue, BlockSyncQueue};
use crate::system::utxo::{UtxoSetReader, UtxoSetWriter};
use crate::system::validation::block::BlockValidator;
use crate::system::validation::transaction::TransactionValidator;
use common::config::node::NodeConfig;
use common::error::{AppError, NetworkError};
use common::{log_node_debug, log_node_error, log_node_info};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct NodeRunning {
    pub(super) cfg: NodeConfig,
    pub(super) blockchain: Arc<dyn Blockchain>,
    pub(super) mempool: Arc<dyn Mempool>,
    pub(super) utxo_set_r: Arc<dyn UtxoSetReader>,
    pub(super) utxo_set_w: Arc<dyn UtxoSetWriter>,
    pub(super) network: Arc<dyn P2PNetworkHandle>,
    pub(super) block_sync_queue: Arc<dyn BlockSyncQueue>,
    pub(super) block_proc_queue: Arc<dyn BlockProcessingQueue>,
    pub(super) block_validator: Arc<dyn BlockValidator>,
    pub(super) tx_validator: Arc<dyn TransactionValidator>,
}

impl NodeRunning {
    pub(super) async fn new(node: NodeStarted) -> Result<Self, AppError> {
        log_node_info!("Node is running...");

        let node = Self {
            cfg: node.cfg,
            blockchain: node.blockchain,
            mempool: node.mempool,
            utxo_set_r: node.utxo_set_rw.0,
            utxo_set_w: node.utxo_set_rw.1,
            network: node.network,
            block_sync_queue: node.block_sync_queue,
            block_proc_queue: node.block_proc_queue,
            block_validator: node.block_validator,
            tx_validator: node.tx_validator,
        };
        Ok(node)
    }

    pub(super) async fn run(
        self,
        mut bus_rx: Box<dyn CommandReceiver>,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), AppError> {
        // Handle Network Block Processing
        tokio::spawn(Self::spawn_block_queue_events_processor_worker_task(
            self.block_proc_queue.clone(),
            self.blockchain.clone(),
            self.block_validator.clone(),
            shutdown_rx,
        ));

        // Handle Bus Events
        loop {
            match bus_rx.receive().await {
                Some(cmd) => match self.handle_bus_event(cmd).await {
                    Ok(HandleBusEventResponse::Continue) => {}
                    Ok(HandleBusEventResponse::Shutdown) => {
                        break;
                    }
                    Err(err) => {
                        log_node_error!("Error handling command: {err}");
                    }
                },
                None => {
                    log_node_info!("Command channel closed!");
                    break;
                }
            }
        }

        // Initiate Graceful Shutdown
        self.terminate(shutdown_tx)?;
        Ok(())
    }

    pub(crate) fn terminate(
        self,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
    ) -> Result<NodeTerminating, AppError> {
        NodeTerminating::terminate(self, shutdown_tx)
    }

    async fn spawn_block_queue_events_processor_worker_task(
        processing_queue: Arc<dyn BlockProcessingQueue>,
        blockchain: Arc<dyn Blockchain>,
        block_validator: Arc<dyn BlockValidator>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) {
        let poll_interval = Duration::from_millis(100); // TODO: pass from cfg

        loop {
            tokio::select! {
                // Await next block
                result = processing_queue.next_ready_block() => {
                    if let Some(block) = result {
                        let height = block.get_height();
                        let hash = block.get_hash();
                        let validated_block = match block_validator.validate_block(block).await {
                            Ok(block) => block,
                            Err(err) => {
                                log_node_error!("Failed to validate block! | Height {:?}: | Hash {:?} | Error: {:?}", height, hash, err);
                                // TODO:
                                // Distinguish between failure causes.
                                // If non-transient, remove the invalid block from the queue.
                                processing_queue.mark_block_failed(&height).await;
                                continue;
                            }
                        };
                        match blockchain.add_block(validated_block).await {
                            Ok(_) => {
                                processing_queue.mark_block_processed(&height).await;
                            }
                            Err(err) => {
                                log_node_error!("Failed to add block! | Height {:?}  | Hash{:?}: | Error: {:?}", height, hash, err);
                                processing_queue.mark_block_failed(&height).await;
                            }
                        }
                    } else {
                        // Exit fast on shutdown signal
                        tokio::select! {
                            _ = shutdown_rx.recv() => break,
                            _ = time::sleep(poll_interval) => {},
                        }
                    }
                }

                // Handle shutdown signal
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        log_node_info!("Block queue events processor worker task exiting....");
    }

    async fn handle_bus_event(
        &self,
        cmd: NodeCommandRequest,
    ) -> Result<HandleBusEventResponse, AppError> {
        log_node_debug!("NodeRunning.handle_event() | cmd:\n{cmd:#?}");

        // TODO: error handling seems sketchy...

        let res = match cmd {
            NodeCommandRequest::BlockchainInitiateGenesis(cfg, responder) => {
                let res = async {
                    let block = NonValidatedBlock::new_genesis(cfg)?;
                    let block = self.block_validator.validate_block(block).await?;
                    self.blockchain.add_block(block).await?;
                    Ok(())
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainHandleMineBlock(block_tpl, responder) => {
                let res = async {
                    let block = NonValidatedBlock::from_template(block_tpl)?;
                    let block = self.block_validator.validate_block(block).await?;
                    self.blockchain.add_block(block.clone()).await?;
                    Ok(block)
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainHandleBlockAppend(block, responder) => {
                let res = async {
                    // TODO: idempotency
                    // TODO: rollback on failure

                    // Consider wrapping these in an external atomic transaction.
                    // Requires an outbox for async P2P side-effects as run_in_transaction() can't be async.
                    self.utxo_set_w.apply_block(&block)?;
                    self.mempool.apply_block(&block).await?;
                    self.blockchain.set_tip(&block).await?;
                    let network_event =
                        NetworkEvent::Gossipsub(GossipsubNetworkEvent::BroadcastNewBlock(block));
                    self.network.publish_network_event(network_event)?;
                    Ok(())
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainGetTipInfo(responder) => {
                let res = async { self.blockchain.get_tip_info().await }.await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainGetBlock(block_hash, responder) => {
                let res = async { self.blockchain.get_canon_block(&block_hash).await }.await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainGetBlockByHeight(block_height, responder) => {
                let res = async {
                    self.blockchain
                        .get_canon_block_by_height(&block_height)
                        .await
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainGetBlocksByHeightRange(height_range, responder) => {
                let res = async {
                    self.blockchain
                        .get_canon_blocks_by_height_range(height_range)
                        .await
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::MempoolPlaceUnconfirmedTransaction(tx, responder) => {
                let res = async {
                    let tx = self.tx_validator.validate_transaction(tx).await?;
                    self.mempool.add_transaction(tx.clone()).await?;
                    Ok(tx)
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::MempoolGetPaginatedTransactions(pagination, responder) => {
                let res = async {
                    let (transactions, count) =
                        self.mempool.get_paginated_transactions(pagination).await?;
                    Ok((transactions, count))
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::MempoolGetUnconfirmedTransactionsByHashes(tx_hashes, responder) => {
                let res = async {
                    let mut transactions = Vec::with_capacity(tx_hashes.len());
                    for tx_hash in &tx_hashes {
                        if let Some(tx) = self.mempool.get_transaction(tx_hash).await {
                            transactions.push(tx);
                        }
                    }
                    if transactions.len() != tx_hashes.len() {
                        return Err(AppError::bad_request(format!(
                            "Transaction count mismatch! Expected {} transaction(s), but only found {}.",
                            tx_hashes.len(),
                            transactions.len(),
                        )));
                    }
                    Ok(transactions)
                }.await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainGetUtxosByOutpoints(outpoints, responder) => {
                let res = { self.utxo_set_r.get_multiple_utxos_by_outpoints(&outpoints) };
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::BlockchainGetUtxos(responder) => {
                let res = { self.utxo_set_r.get_multiple_utxos() };
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::P2PHandleReceiveBlockchainTipInfo(
                origin_peer_id,
                block_info,
                responder,
            ) => {
                let res = async {
                    #[allow(clippy::collapsible_if)]
                    if let Some(tip_info) = block_info {
                        if let Some(unknown_heights) =
                            self.blockchain.get_unknown_block_heights(tip_info).await?
                        {
                            // Step trait is unstable...
                            for height in
                                unknown_heights.start().as_u64()..=unknown_heights.end().as_u64()
                            {
                                let height = height.into();
                                if !self.block_sync_queue.is_in_progress(&height).await {
                                    self.block_sync_queue
                                        .request_block(height, origin_peer_id.clone())
                                        .await?;
                                }
                            }
                        }
                    }
                    Ok(())
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::P2PHandleReceiveBlocks(origin_peer_id, blocks, responder) => {
                let res = async {
                    for block in blocks {
                        if self.blockchain.has_canon_block(&block.get_hash()).await? {
                            continue;
                        }
                        self.block_sync_queue
                            .on_block_received(block, origin_peer_id.clone())
                            .await
                    }
                    Ok(())
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::NetworkGetSelfInfo(responder) => {
                let res = async {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let network_event = NetworkEvent::GetSelfInfo(tx);
                    self.network.publish_network_event(network_event)?;
                    let peers = rx.await.map_err(|err| {
                        AppError::internal_with_private(
                            "Failed to retrieve network info!",
                            err.to_string(),
                        )
                    })?;
                    Ok(peers)
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::NetworkGetPeers(responder) => {
                let res = async {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let network_event = NetworkEvent::GetPeers(tx);
                    self.network.publish_network_event(network_event)?;
                    let peers = rx.await.map_err(|err| {
                        AppError::internal_with_private(
                            "Failed to retrieve network peers!",
                            err.to_string(),
                        )
                    })?;
                    Ok(peers)
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::NetworkAddPeer(network_address, responder) => {
                let res = async {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let network_event = NetworkEvent::AddPeer(network_address, tx);
                    self.network.publish_network_event(network_event)?;
                    let res = rx.await.map_err(|err| {
                        AppError::Network(NetworkError::PeerConnectionFailed {
                            reason: err.to_string(),
                        })
                    })?;
                    Ok(res)
                }
                .await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::ProxyForwardNetworkEvent(event, responder) => {
                let res = async { self.network.publish_network_event(event) }.await;
                responder.respond(res);
                HandleBusEventResponse::Continue
            }
            NodeCommandRequest::RequestNodeShutdown => HandleBusEventResponse::Shutdown,
        };
        Ok(res)
    }
}

enum HandleBusEventResponse {
    Continue,
    Shutdown,
}
