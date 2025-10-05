use crate::encode::{TryDecode, TryEncode};
use crate::entities::block::BlockHeight;
use crate::ext::AppErrorConvertibleDomain;
use crate::types::network::NetworkPeerId;
use bincode::{Decode, Encode};
use common::error::AppError;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct TaliroNetworkEvent {
    header: TaliroNetworkEventHeader,
    data: TaliroNetworkData,
}

#[derive(Debug, Clone)]
pub struct TaliroNetworkEventHeader {
    recipient_peer_id: NetworkPeerId,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum TaliroNetworkData {
    GetBlockchainTip,
    GetBlockByHeight(BlockHeight),
    GetBlocksByHeightRange(RangeInclusive<BlockHeight>),
}

impl TaliroNetworkEventHeader {
    pub fn get_recipient_peer_id(&self) -> NetworkPeerId {
        self.recipient_peer_id.clone()
    }
}

impl TaliroNetworkEvent {
    pub fn new(recipient_peer_id: NetworkPeerId, data: TaliroNetworkData) -> Self {
        let header = TaliroNetworkEventHeader { recipient_peer_id };
        Self { header, data }
    }

    pub fn get_header(&self) -> &TaliroNetworkEventHeader {
        &self.header
    }

    pub fn get_data(&self) -> &TaliroNetworkData {
        &self.data
    }
}

impl TryEncode for TaliroNetworkData {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for TaliroNetworkData {
    fn try_decode(data: &[u8]) -> Result<Self, AppError>
    where
        Self: Sized,
    {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}
