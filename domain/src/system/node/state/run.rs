use crate::system::blockchain::Blockchain;
use crate::system::mempool::Mempool;
use crate::system::network::P2PNetworkHandle;
use crate::system::node::cmd::CommandReceiver;
use crate::system::node::cmd::handlers::CommandDispatcher;
use crate::system::node::state::exit::NodeTerminating;
use crate::system::node::state::start::NodeStarted;
use crate::system::queue::BlockProcessingQueue;
use crate::system::utxo::{UtxoSetReader, UtxoSetWriter};
use crate::system::validation::block::BlockValidator;
use common::config::node::NodeConfig;
use common::error::AppError;
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
    pub(super) block_validator: Arc<dyn BlockValidator>,
    block_proc_queue: Arc<dyn BlockProcessingQueue>,
    cmd_dispatcher: CommandDispatcher,
}

impl NodeRunning {
    pub(super) fn new(node: NodeStarted) -> Self {
        log_node_info!("Node is running...");

        Self {
            cfg: node.cfg,
            blockchain: node.blockchain,
            mempool: node.mempool,
            utxo_set_r: node.utxo_set_rw.0,
            utxo_set_w: node.utxo_set_rw.1,
            network: node.network,
            block_validator: node.block_validator,
            block_proc_queue: node.block_proc_queue,
            cmd_dispatcher: node.cmd_dispatcher,
        }
    }

    pub(super) async fn run(
        self,
        mut cmd_rx: Box<dyn CommandReceiver>,
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

        // Handle Command Events
        loop {
            match cmd_rx.receive().await {
                Some(cmd) => {
                    log_node_debug!("NodeRunning: Received command");

                    match self.cmd_dispatcher.dispatch(cmd).await {
                        Ok(response) => match response {
                            crate::system::node::cmd::handlers::CommandHandlerControlFlow::Continue => {}
                            crate::system::node::cmd::handlers::CommandHandlerControlFlow::Shutdown => {
                                log_node_info!("Shutdown command received");
                                break;
                            }
                        },
                        Err(err) => {
                            log_node_error!("Error handling command: {err}");
                            // Continue processing other commands despite errors
                        }
                    }
                }
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
}
