use async_trait::async_trait;
use bincode::{Decode, Encode};
use domain::entities::block::{Block, BlockHeight};
use domain::types::hash::Hash;
use libp2p::{
    futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, io},
    request_response,
};
use std::ops::RangeInclusive;

// Notes:
// On a protocol level, validatable entities (like blocks) are sent/received as fully validated struct variants.
// Validation is expected to be performed before sending, and after receiving.

/// `Taliro` blockchain's P2P protocol.<br />
/// Validatable entities are exchanged as fully validated struct variants.<br />
/// Peers are expected to explicitly invalidate and re-validate incoming data.
#[derive(Clone)]
pub(crate) struct TaliroProtocol();

impl AsRef<str> for TaliroProtocol {
    fn as_ref(&self) -> &str {
        "/kon14/taliro/0.1.0"
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub(crate) enum TaliroProtocolRequest {
    GetBlockchainTip,
    GetBlockByHeight(BlockHeight),
    GetBlocksByHeightRange(RangeInclusive<BlockHeight>),
}

#[derive(Debug, Clone, Encode, Decode)]
pub(crate) enum TaliroProtocolResponse {
    BlockchainTip(Option<(Hash, BlockHeight)>),
    GetBlockByHeight(Option<Block>),
    GetBlocksByHeightRange(Vec<Block>),
}

#[derive(Clone, Default)]
pub(crate) struct BlockchainProtocolExchangeCodec;

#[async_trait]
impl request_response::Codec for BlockchainProtocolExchangeCodec {
    type Protocol = TaliroProtocol;
    type Request = TaliroProtocolRequest;
    type Response = TaliroProtocolResponse;

    async fn read_request<T>(&mut self, _: &TaliroProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let config = bincode::config::standard();
        bincode::decode_from_slice(&vec, config)
            .map(|(request, _)| request)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    async fn read_response<T>(
        &mut self,
        _: &TaliroProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let config = bincode::config::standard();
        bincode::decode_from_slice(&vec, config)
            .map(|(response, _)| response)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    async fn write_request<T>(
        &mut self,
        _: &TaliroProtocol,
        io: &mut T,
        request: TaliroProtocolRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(&request, config)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        write_length_prefixed(io, &encoded).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &TaliroProtocol,
        io: &mut T,
        response: TaliroProtocolResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(&response, config)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

        write_length_prefixed(io, &encoded).await?;
        io.close().await?;
        Ok(())
    }
}

async fn read_length_prefixed<T: AsyncRead + Unpin>(
    io: &mut T,
    max_size: usize,
) -> io::Result<Vec<u8>> {
    let mut length_bytes = [0u8; 4];
    io.read_exact(&mut length_bytes).await?;
    let length = u32::from_be_bytes(length_bytes) as usize;

    if length > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Message too large",
        ));
    }

    let mut buffer = vec![0u8; length];
    io.read_exact(&mut buffer).await?;
    Ok(buffer)
}

async fn write_length_prefixed<T: AsyncWrite + Unpin>(io: &mut T, data: &[u8]) -> io::Result<()> {
    let length = data.len() as u32;
    io.write_all(&length.to_be_bytes()).await?;
    io.write_all(data).await?;
    Ok(())
}
