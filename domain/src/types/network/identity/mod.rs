use crate::encode::{TryDecode, TryEncode};
use crate::ext::AppErrorConvertibleDomain;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bincode::{Decode, Encode};
use common::error::AppError;

/// An opaque wrapper over the serialized network identity keypair.
/// The actual keypair format is determined by the underlying networking infrastructure.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct NetworkIdentityKeypair(String);

impl NetworkIdentityKeypair {
    pub fn from_proto_bytes(bytes: Vec<u8>) -> Self {
        // Encode to base64
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn from_base64(base64: String) -> Self {
        Self(base64)
    }

    pub fn as_proto_bytes(&self) -> Result<Vec<u8>, AppError> {
        // Decode from base64
        let bytes = URL_SAFE_NO_PAD
            .decode(self.0.clone())
            .map_err(|err| AppError::internal_with_private("Fail", err.to_string()))?;
        Ok(bytes)
    }

    pub fn as_base64(&self) -> String {
        self.0.clone()
    }
}

impl TryEncode for NetworkIdentityKeypair {
    fn try_encode(&self) -> Result<Vec<u8>, AppError> {
        let config = bincode::config::standard();
        let data = bincode::encode_to_vec(self, config).to_app_error()?;
        Ok(data)
    }
}

impl TryDecode for NetworkIdentityKeypair {
    fn try_decode(data: &[u8]) -> Result<Self, AppError> {
        let config = bincode::config::standard();
        let (data, _): (Self, usize) = bincode::decode_from_slice(data, config).to_app_error()?;
        Ok(data)
    }
}
