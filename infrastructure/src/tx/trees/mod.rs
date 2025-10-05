use sled::Tree;

#[derive(Clone)]
pub(crate) enum SledTxTrees {
    BlockchainAppendBlock(SledTxBlockchainAppendBlockTrees),
    UtxoSetAppendBlock(SledTxUtxoSetAppendBlockTrees),
}

#[derive(Clone)]
pub(crate) struct SledTxBlockchainAppendBlockTrees {
    pub(crate) blocks_tree: Tree,
    pub(crate) heights_tree: Tree,
    pub(crate) outbox_unprocessed_tree: Tree,
}

#[derive(Clone)]
pub(crate) struct SledTxUtxoSetAppendBlockTrees {
    pub(crate) utxo_tree: Tree,
}
