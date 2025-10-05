#[cfg(test)]
mod tests;

use bincode::{Decode, Encode};
use common::error::{AppError, CryptographicError};
use std::fmt;

const BYTES_LENGTH: usize = 32;

#[derive(Clone, Encode, Decode, PartialEq, Eq, std::hash::Hash)]
pub struct Hash([u8; BYTES_LENGTH]);

impl Hash {
    const STRING_CHAR_LENGTH: usize = 64;

    pub fn new(data: [u8; BYTES_LENGTH]) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8; BYTES_LENGTH] {
        &self.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self)
    }
}

impl AsRef<[u8; BYTES_LENGTH]> for Hash {
    fn as_ref(&self) -> &[u8; BYTES_LENGTH] {
        &self.0
    }
}

pub trait Hashable {
    fn hash(&self) -> Hash;
}

pub trait TryHashable {
    fn try_hash(&self) -> Result<Hash, AppError>;
}

impl TryFrom<&str> for Hash {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.len() != Self::STRING_CHAR_LENGTH {
            return Err(AppError::Cryptographic(
                CryptographicError::HashLengthMismatch {
                    expected: Self::STRING_CHAR_LENGTH,
                    actual: s.len(),
                },
            ));
        }

        let mut bytes = [0u8; BYTES_LENGTH];
        for i in 0..BYTES_LENGTH {
            let byte_str = &s[2 * i..2 * i + 2];
            match u8::from_str_radix(byte_str, 16) {
                Ok(b) => bytes[i] = b,
                Err(_) => {
                    return Err(AppError::Cryptographic(
                        CryptographicError::DecodingFailed {
                            reason: "Hash conversion failed!".to_string(),
                        },
                    ));
                }
            }
        }

        Ok(Hash(bytes))
    }
}
