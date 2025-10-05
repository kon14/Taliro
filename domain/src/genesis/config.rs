use crate::entities::transaction::TransactionAmount;
use crate::types::sign::PublicKey;
use crate::types::time::DateTime;

#[derive(Debug)]
pub struct GenesisConfig {
    pub(crate) utxos: Vec<GenesisConfigUtxoFunds>,
    pub(crate) timestamp: DateTime,
}

#[derive(Debug)]
pub struct GenesisConfigUtxoFunds {
    pub(crate) wallet_pub_key: PublicKey,
    pub(crate) amount: TransactionAmount,
}

impl GenesisConfig {
    pub fn new_unchecked(utxos: Vec<GenesisConfigUtxoFunds>, timestamp: DateTime) -> Self {
        Self { utxos, timestamp }
    }
}

impl GenesisConfigUtxoFunds {
    pub fn new_unchecked(wallet_pub_key: PublicKey, amount: TransactionAmount) -> Self {
        Self {
            wallet_pub_key,
            amount,
        }
    }
}
