use crate::encode::{TryDecode, TryEncode};
use crate::entities::block::Block;
use crate::ext::AppErrorConvertibleDomain;
use crate::types::time::DateTime;
use bincode::enc::Encoder;
use bincode::error::EncodeError;
use bincode::{Decode, Encode};
use common::error::AppError;
use uuid::Uuid;

#[derive(Clone, Debug, Encode, Decode)]
pub enum OutboxEvent {
    BlockchainAppendBlock(Block),
}

impl OutboxEvent {
    pub fn get_event_type(&self) -> &'static str {
        match self {
            OutboxEvent::BlockchainAppendBlock(_) => "BlockchainAppendBlock",
        }
    }
}

#[derive(Clone, Debug)]
pub struct OutboxEntry {
    pub(crate) id: Uuid,
    pub(crate) event: OutboxEvent,
    pub(crate) created_at: DateTime,
    pub(crate) processed: bool,
}

impl OutboxEntry {
    pub(crate) fn new(event: OutboxEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            created_at: DateTime::now(),
            processed: false,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_event(&self) -> &OutboxEvent {
        &self.event
    }

    pub fn mark_processed(&mut self) {
        self.processed = true;
    }
}

impl bincode::Encode for OutboxEntry {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.id.as_bytes().encode(encoder)?;
        self.event.encode(encoder)?;
        self.created_at.encode(encoder)?;
        self.processed.encode(encoder)?;
        Ok(())
    }
}

impl bincode::Decode<()> for OutboxEntry {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id_bytes: [u8; 16] = Decode::decode(decoder)?;
        let id = Uuid::from_bytes(id_bytes);
        let event = Decode::decode(decoder)?;
        let created_at = Decode::decode(decoder)?;
        let processed = bool::decode(decoder)?;
        Ok(Self {
            id,
            event,
            created_at,
            processed,
        })
    }
}

impl TryEncode for OutboxEntry {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for OutboxEntry {
    fn try_decode(data: &[u8]) -> Result<Self, AppError> {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}
