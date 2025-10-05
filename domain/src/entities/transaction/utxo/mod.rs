use crate::entities::transaction::{TransactionOutPoint, TransactionOutput};

#[derive(Clone, Debug)]
pub struct Utxo {
    outpoint: TransactionOutPoint,
    output: TransactionOutput,
}

impl Utxo {
    pub fn new(outpoint: TransactionOutPoint, output: TransactionOutput) -> Utxo {
        Utxo { outpoint, output }
    }

    pub fn get_outpoint(&self) -> &TransactionOutPoint {
        &self.outpoint
    }

    pub fn get_output(&self) -> &TransactionOutput {
        &self.output
    }
}
