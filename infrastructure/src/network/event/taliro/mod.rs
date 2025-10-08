use super::super::{
    behavior::AppNetworkBehavior,
    protocol::{TaliroProtocolRequest, TaliroProtocolResponse},
};
use common::{log_net_gs_error, log_net_gs_trace, log_net_taliro_error, log_net_taliro_trace};
use domain::system::node::cmd::{CommandResponderFactory, CommandSender};
use domain::types::network::NetworkPeerId;
use libp2p::{Swarm, request_response};
use std::sync::Arc;

pub(super) async fn handle_taliro_event(
    event: request_response::Event<TaliroProtocolRequest, TaliroProtocolResponse>,
    swarm: &mut Swarm<AppNetworkBehavior>,
    cmd_tx: &Arc<dyn CommandSender>,
    cmd_tx_res_factory: &Arc<dyn CommandResponderFactory>,
) {
    #[allow(clippy::single_match)]
    match event {
        request_response::Event::Message { peer, message, .. } => match message {
            request_response::Message::Request { .. } => {
                handle_taliro_request_message_event(
                    peer,
                    message,
                    swarm,
                    cmd_tx,
                    cmd_tx_res_factory,
                )
                .await
            }
            request_response::Message::Response { .. } => {
                handle_taliro_response_message_event(peer, message, cmd_tx, cmd_tx_res_factory)
                    .await
            }
        },
        _ => {}
    }
}

async fn handle_taliro_request_message_event(
    peer_id: libp2p::PeerId,
    event: request_response::Message<TaliroProtocolRequest, TaliroProtocolResponse>,
    swarm: &mut Swarm<AppNetworkBehavior>,
    cmd_tx: &Arc<dyn CommandSender>,
    cmd_tx_res_factory: &Arc<dyn CommandResponderFactory>,
) {
    let request_response::Message::Request {
        request, channel, ..
    } = event
    else {
        return;
    };

    log_net_gs_trace!("Received request from {peer_id}:\n{:#?}", request);

    let response = match request {
        TaliroProtocolRequest::GetBlockchainTip => {
            let (command, res_fut) = cmd_tx_res_factory.build_blk_cmd_get_tip_info();
            let Ok(_) = cmd_tx.send(command).await else {
                log_net_gs_error!("Failed to send GetBlockchainTipInfo command.");
                return;
            };
            let Ok(block_info) = res_fut.await else {
                log_net_gs_error!("GetBlockchainTipInfo command failed.");
                return;
            };
            TaliroProtocolResponse::BlockchainTip(block_info)
        }
        TaliroProtocolRequest::GetBlockByHeight(height) => {
            let (command, res_fut) = cmd_tx_res_factory.build_blk_cmd_get_block_by_height(height);
            let Ok(_) = cmd_tx.send(command).await else {
                log_net_gs_error!("Failed to send GetBlockchainBlockByHeight command.");
                return;
            };
            let Ok(block) = res_fut.await else {
                log_net_gs_error!("GetBlockchainBlockByHeight command failed.");
                return;
            };
            TaliroProtocolResponse::GetBlockByHeight(block)
        }
        TaliroProtocolRequest::GetBlocksByHeightRange(range) => {
            let (command, res_fut) =
                cmd_tx_res_factory.build_blk_cmd_get_blocks_by_height_range(range);
            let Ok(_) = cmd_tx.send(command).await else {
                log_net_gs_error!("Failed to send GetBlockchainBlocksByHeightRange command.");
                return;
            };
            let Ok(blocks) = res_fut.await else {
                log_net_gs_error!("GetBlockchainBlocksByHeightRange command failed.");
                return;
            };
            TaliroProtocolResponse::GetBlocksByHeightRange(blocks)
        }
    };

    if let Err(response) = swarm
        .behaviour_mut()
        .get_blockchain_mut()
        .send_response(channel, response)
    {
        log_net_gs_error!(
            "Failed to send response to {peer_id}: response was {:?}",
            response
        );
    }
}

async fn handle_taliro_response_message_event(
    peer_id: libp2p::PeerId,
    event: request_response::Message<TaliroProtocolRequest, TaliroProtocolResponse>,
    cmd_tx: &Arc<dyn CommandSender>,
    cmd_tx_res_factory: &Arc<dyn CommandResponderFactory>,
) {
    let request_response::Message::Response { response, .. } = event else {
        return;
    };

    log_net_taliro_trace!("Received response from {peer_id}:\n{:#?}", response);
    // Already a PeerId
    let peer_id = NetworkPeerId::_new_validated(peer_id.to_bytes(), peer_id.to_string());

    match response {
        TaliroProtocolResponse::BlockchainTip(block_info) => {
            let (command, res_fut) =
                cmd_tx_res_factory.build_p2p_cmd_receive_blockchain_tip_info(peer_id, block_info);
            let Ok(_) = cmd_tx.send(command).await else {
                log_net_taliro_error!("Failed to send ReceiveBlockchainTipInfo command.");
                return;
            };
            if let Err(err) = res_fut.await {
                log_net_taliro_error!("ReceiveBlockchainTipInfo command failed: {err}");
            }
        }
        TaliroProtocolResponse::GetBlockByHeight(block) => {
            let Some(block) = block else {
                return;
            };
            let block = block.invalidate();
            let (command, res_fut) =
                cmd_tx_res_factory.build_p2p_cmd_receive_blocks(peer_id, vec![block]);
            let Ok(_) = cmd_tx.send(command).await else {
                log_net_taliro_error!("Failed to send ReceiveBlocksByHeightRange command.");
                return;
            };
            if let Err(err) = res_fut.await {
                log_net_taliro_error!("ReceiveBlocksByHeightRange command failed: {err}");
            }
        }
        TaliroProtocolResponse::GetBlocksByHeightRange(blocks) => {
            let blocks = blocks.into_iter().map(|b| b.invalidate()).collect();
            let (command, res_fut) =
                cmd_tx_res_factory.build_p2p_cmd_receive_blocks(peer_id, blocks);
            let Ok(_) = cmd_tx.send(command).await else {
                log_net_taliro_error!("Failed to send ReceiveBlocksByHeightRange command.");
                return;
            };
            if let Err(err) = res_fut.await {
                log_net_taliro_error!("ReceiveBlocksByHeightRange command failed: {err}");
            }
        }
    }
}
