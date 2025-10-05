use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Invalid multiaddr format: {addr}")]
    InvalidMultiaddr { addr: String },

    #[error("Peer connection failed: {reason}")]
    PeerConnectionFailed { reason: String },

    #[error("Protocol error: {reason}")]
    ProtocolError { reason: String },
}
