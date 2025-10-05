use crate::types::hash::{Hash, Hashable};
use crate::types::sign::PublicKey;
use bincode::{Decode, Encode};
use common::error::AppError;
use std::fmt;
use std::str::FromStr;

/// Represents a wallet address as a hash of its public key.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct WalletAddress(Hash);

impl From<&PublicKey> for WalletAddress {
    fn from(pub_key: &PublicKey) -> Self {
        let hash = pub_key.hash();
        WalletAddress(hash)
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for WalletAddress {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = Hash::try_from(s).map_err(|err| {
            AppError::internal_with_private("Couldn't parse wallet address.", err.to_string())
        })?;
        Ok(WalletAddress(hash))
    }
}
