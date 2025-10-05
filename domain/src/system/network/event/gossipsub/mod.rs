use crate::encode::{TryDecode, TryEncode};
use crate::entities::block::Block;
use crate::ext::AppErrorConvertibleDomain;
use bincode::{Decode, Encode};
use common::error::AppError;

#[derive(Debug, Clone, Encode, Decode)]
pub enum GossipsubNetworkEvent {
    BroadcastNewBlock(Block),
}

impl TryEncode for GossipsubNetworkEvent {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for GossipsubNetworkEvent {
    fn try_decode(data: &[u8]) -> Result<Self, AppError>
    where
        Self: Sized,
    {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}
