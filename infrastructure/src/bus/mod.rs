use crate::ext::TryFromStrDomainNetworkAddressExtInfrastructure;
use async_trait::async_trait;
use common::error::AppError;
use common::params::PaginationParams;
use domain::entities::block::{Block, BlockHeight, BlockTemplate, NonValidatedBlock};
use domain::entities::transaction::{
    NonValidatedTransaction, Transaction, TransactionOutPoint, Utxo,
};
use domain::genesis::config::GenesisConfig;
use domain::system::network::event::{AddPeerResponse, NetworkEvent};
use domain::system::node::bus::{
    CommandReceiver, CommandResponder, CommandResponderFactory, CommandSender, NodeCommandRequest,
};
use domain::types::hash::Hash;
use domain::types::network::{NetworkAddress, NetworkIdentityKeypair, NetworkPeerId};
use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::pin::Pin;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Debug)]
pub struct NodeCommandSender(mpsc::Sender<NodeCommandRequest>);

pub struct NodeCommandReceiver(mpsc::Receiver<NodeCommandRequest>);

pub struct NodeCommandOneshotResponderFactory;

pub fn build_channel(buffer_size: usize) -> (NodeCommandSender, NodeCommandReceiver) {
    let (bus_tx, bus_rx) = mpsc::channel(buffer_size);
    let sender = NodeCommandSender(bus_tx);
    let receiver = NodeCommandReceiver(bus_rx);
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

#[derive(Debug)]
pub struct TokioResponder<T: Debug>(oneshot::Sender<T>);

impl<T: Send + Debug> CommandResponder<T> for TokioResponder<T> {
    fn respond(self: Box<Self>, value: T) {
        let _ = self.0.send(value);
    }
}

pub fn create_responder<T>() -> (
    Box<dyn CommandResponder<T> + Send>,
    impl Future<Output = T> + Send,
)
where
    T: Send + Debug + 'static,
{
    let (tx, rx) = oneshot::channel();
    let responder = Box::new(TokioResponder(tx)) as Box<dyn CommandResponder<T> + Send>;
    let fut = async move {
        rx.await.expect("Responder dropped without responding!") // TODO: handle error properly
    };
    (responder, fut)
}

#[derive(Debug)]
pub struct NodeCommandResponderFactory;

impl CommandResponderFactory for NodeCommandResponderFactory {
    fn build_blk_cmd_init_genesis(
        &self,
        cfg: GenesisConfig,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder =
            Box::new(TokioResponder(tx)) as Box<dyn CommandResponder<Result<(), AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainInitiateGenesis(cfg, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_blk_cmd_handle_mine_block(
        &self,
        block_tpl: BlockTemplate,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Block, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Block, AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainHandleMineBlock(block_tpl, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_blk_cmd_handle_block_append(
        &self,
        block: Block,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder =
            Box::new(TokioResponder(tx)) as Box<dyn CommandResponder<Result<(), AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainHandleBlockAppend(block, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_blk_cmd_get_tip_info(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<(Hash, BlockHeight)>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Option<(Hash, BlockHeight)>, AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainGetTipInfo(responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_blk_cmd_get_block(
        &self,
        block_hash: Hash,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<Block>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Option<Block>, AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainGetBlock(block_hash, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_blk_cmd_get_block_by_height(
        &self,
        height: BlockHeight,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Option<Block>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Option<Block>, AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainGetBlockByHeight(height, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_blk_cmd_get_blocks_by_height_range(
        &self,
        height_range: RangeInclusive<BlockHeight>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Block>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Vec<Block>, AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainGetBlocksByHeightRange(height_range, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_mp_cmd_place_transaction(
        &self,
        transaction: NonValidatedTransaction,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Transaction, AppError>> + Send>;

        let command = NodeCommandRequest::MempoolPlaceTransaction(transaction, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_mp_get_paginated_transactions(
        &self,
        pagination: PaginationParams,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(Vec<Transaction>, usize), AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<(Vec<Transaction>, usize), AppError>> + Send>;

        let command = NodeCommandRequest::MempoolGetPaginatedTransactions(pagination, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_mp_cmd_get_transactions_by_hashes(
        &self,
        tx_hashes: Vec<Hash>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Vec<Transaction>, AppError>> + Send>;

        let command = NodeCommandRequest::MempoolGetTransactionsByHashes(tx_hashes, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_blk_get_utxos_by_outpoints(
        &self,
        outpoints: Vec<TransactionOutPoint>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Utxo>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Vec<Utxo>, AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainGetUtxosByOutpoints(outpoints, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_cmd_get_utxos(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<Utxo>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Vec<Utxo>, AppError>> + Send>;

        let command = NodeCommandRequest::BlockchainGetUtxos(responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

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
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<
                dyn CommandResponder<
                        Result<(NetworkIdentityKeypair, Vec<NetworkAddress>), AppError>,
                    > + Send,
            >;

        let command = NodeCommandRequest::NetworkGetSelfInfo(responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_net_cmd_get_peers(
        &self,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<Vec<NetworkAddress>, AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<Vec<NetworkAddress>, AppError>> + Send>;

        let command = NodeCommandRequest::NetworkGetPeers(responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_net_cmd_add_peer(
        &self,
        multiaddr_str: String,
    ) -> Result<
        (
            NodeCommandRequest,
            Pin<Box<dyn Future<Output = Result<AddPeerResponse, AppError>> + Send>>,
        ),
        AppError,
    > {
        let network_address = NetworkAddress::try_from_str(&multiaddr_str)?;

        let (tx, rx) = oneshot::channel();
        let responder = Box::new(TokioResponder(tx))
            as Box<dyn CommandResponder<Result<AddPeerResponse, AppError>> + Send>;
        let command = NodeCommandRequest::NetworkAddPeer(network_address, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        Ok((command, fut))
    }

    fn build_p2p_cmd_receive_blockchain_tip_info(
        &self,
        origin_peer_id: NetworkPeerId,
        block_info: Option<(Hash, BlockHeight)>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder =
            Box::new(TokioResponder(tx)) as Box<dyn CommandResponder<Result<(), AppError>> + Send>;

        let command = NodeCommandRequest::P2PHandleReceiveBlockchainTipInfo(
            origin_peer_id,
            block_info,
            responder,
        );

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_p2p_cmd_receive_blocks(
        &self,
        origin_peer_id: NetworkPeerId,
        blocks: Vec<NonValidatedBlock>,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder =
            Box::new(TokioResponder(tx)) as Box<dyn CommandResponder<Result<(), AppError>> + Send>;

        let command = NodeCommandRequest::P2PHandleReceiveBlocks(origin_peer_id, blocks, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }

    fn build_proxy_cmd_forward_network_event(
        &self,
        event: NetworkEvent,
    ) -> (
        NodeCommandRequest,
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    ) {
        let (tx, rx) = oneshot::channel();
        let responder =
            Box::new(TokioResponder(tx)) as Box<dyn CommandResponder<Result<(), AppError>> + Send>;

        let command = NodeCommandRequest::ProxyForwardNetworkEvent(event, responder);

        let fut = Box::pin(async move {
            rx.await
                .expect("Responder dropped without sending response!") // TODO: handle error properly
        });

        (command, fut)
    }
}
