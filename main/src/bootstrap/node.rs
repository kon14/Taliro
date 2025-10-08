use application::outbox::OutboxRelay;
use application::queue::{DefaultBlockProcessingQueue, DefaultBlockSyncQueue};
use application::storage::Storage;
use common::config::node::NodeConfig;
use common::error::AppError;
use domain::entities::block::BlockHeight;
use domain::system::network::P2PNetworkEngine;
use domain::system::node;
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::system::queue::{BlockProcessingQueue, BlockSyncQueue};
use std::sync::Arc;

pub(crate) async fn build_node(
    config: NodeConfig,
    storage: Box<dyn Storage>,
    network: Box<dyn P2PNetworkEngine>,
    cmd_tx: Arc<dyn CommandSender>,
    cmd_tx_res_factory: Arc<dyn CommandResponderFactory>,
) -> Result<
    (
        node::NodeInitialized,
        OutboxRelay,
        Arc<dyn BlockSyncQueue>,
        Arc<dyn BlockProcessingQueue>,
    ),
    AppError,
> {
    // Access Storage Repositories
    let blockchain_repo = storage.get_blockchain_repo();
    let utxo_repo = storage.get_utxo_repo();
    let outbox_repo = storage.get_outbox_repo();

    // Build Outbox Relay
    let outbox_relay = OutboxRelay::new(
        outbox_repo.clone(),
        cmd_tx.clone(),
        cmd_tx_res_factory.clone(),
    );

    // Build Blockchain Node
    let node = node::build_node(config, blockchain_repo, utxo_repo, outbox_repo, network).await?;

    // Initialize Block Sync and Processing Queues
    let blockchain_tip = node.get_tip_info().await?;
    let next_expected_height = blockchain_tip.map_or(BlockHeight::genesis(), |info| info.1.next());
    let block_proc_queue = Arc::new(DefaultBlockProcessingQueue::new(next_expected_height));
    let block_sync_queue = Arc::new(DefaultBlockSyncQueue::new(
        block_proc_queue.clone(),
        cmd_tx.clone(),
        cmd_tx_res_factory.clone(),
    ));

    Ok((node, outbox_relay, block_sync_queue, block_proc_queue))
}
