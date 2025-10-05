use crate::types::hash::{Hash, Hashable};
use bincode::config::Configuration;
use bincode::de::Decoder;
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{Decode, Encode};
use blake2::{Blake2b512, Digest};
use common::error::AppError;
use ed25519_dalek::Signer;
use rand_core::OsRng;
use std::fmt;
use std::str::FromStr;

#[allow(unused)]
const PUBLIC_KEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

#[derive(Clone, Debug)]
pub struct Signature(ed25519_dalek::Signature);

#[derive(Clone, Debug)]
pub struct PrivateKey(ed25519_dalek::SigningKey);

#[derive(Clone, Debug)]
pub struct PublicKey(ed25519_dalek::VerifyingKey);

impl PublicKey {
    #[allow(unused)]
    pub(crate) fn as_bytes(&self) -> &[u8; PUBLIC_KEY_LENGTH] {
        self.0.as_bytes()
    }
}

impl Hashable for PublicKey {
    fn hash(&self) -> Hash {
        let hash = Blake2b512::digest(self.0.as_bytes());
        let mut addr = [0u8; 32];
        addr.copy_from_slice(&hash[..32]);
        Hash::new(addr)
    }
}

impl PrivateKey {
    pub fn generate() -> Self {
        let sk = ed25519_dalek::SigningKey::generate(&mut OsRng);
        PrivateKey(sk)
    }

    pub fn get_public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }

    #[allow(unused)]
    pub(crate) fn sign(&self, msg: &[u8]) -> Signature {
        Signature(self.0.sign(msg))
    }
}

impl Encode for Signature {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        // ed25519_dalek::Signature is 64 bytes, serialize as fixed array
        self.0.to_bytes().encode(encoder)
    }
}

impl Decode<Configuration> for Signature {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bytes: [u8; SIGNATURE_LENGTH] = Decode::decode(decoder)?;
        let sig = ed25519_dalek::Signature::from_bytes(&bytes);
        // TODO: no validation yet...
        Ok(Signature(sig))
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.to_bytes()))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_bytes()))
    }
}

impl FromStr for PrivateKey {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|err| {
            AppError::bad_request_with_private("Couldn't parse private key.", err.to_string())
        })?;
        let byte_array: [u8; 32] = bytes.try_into().map_err(|_| {
            AppError::bad_request("Invalid private key length (expected 32 bytes).")
        })?;
        let sk = ed25519_dalek::SigningKey::from_bytes(&byte_array);
        Ok(PrivateKey(sk))
    }
}

impl FromStr for PublicKey {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|err| {
            AppError::bad_request_with_private("Couldn't parse public key.", err.to_string())
        })?;
        let byte_array: [u8; 32] = bytes
            .try_into()
            .map_err(|_| AppError::bad_request("Invalid public key length (expected 32 bytes)."))?;
        let vk = ed25519_dalek::VerifyingKey::from_bytes(&byte_array).map_err(|err| {
            AppError::bad_request_with_private("Invalid public key.", err.to_string())
        })?;
        Ok(PublicKey(vk))
    }
}
