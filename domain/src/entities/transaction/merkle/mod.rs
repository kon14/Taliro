use crate::entities::transaction::{NonValidatedTransaction, Transaction};
use crate::types::hash::Hash;
use bincode::{Decode, Encode};
use blake2::{Blake2b512, Digest};
use common::error::AppError;

struct MerkleTree {
    nodes: Vec<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct TransactionsMerkleRoot(Hash);

impl TransactionsMerkleRoot {
    /// Constructs a new [`TransactionsMerkleRoot`] from a slice of transactions.<br />
    /// The transaction order **must be preserved** for deterministic outputs!<br />
    /// Returns an error if the input slice is empty.
    pub fn new(transactions: &[Transaction]) -> Result<Self, AppError> {
        let tx_leaves = transactions
            .iter()
            .map(|tx| tx.get_hash())
            .collect::<Vec<_>>();
        let merkle_tree = MerkleTree::new(&tx_leaves)?;
        let hash = merkle_tree.get_root().clone();
        Ok(Self(hash))
    }

    /// Constructs a new [`TransactionsMerkleRoot`] from a slice of non-validated transactions.<br />
    /// The transaction order **must be preserved** for deterministic outputs!<br />
    /// Returns an error if the input slice is empty.
    pub fn new_non_validated(transactions: &[NonValidatedTransaction]) -> Result<Self, AppError> {
        let tx_leaves = transactions
            .iter()
            .map(|tx| tx.get_hash())
            .collect::<Vec<_>>();
        let merkle_tree = MerkleTree::new(&tx_leaves)?;
        let hash = merkle_tree.get_root().clone();
        Ok(Self(hash))
    }

    pub fn inner(&self) -> &Hash {
        &self.0
    }
}

impl MerkleTree {
    /// Constructs a new [`MerkleTree`] from a slice of leaf hashes.<br />
    /// The order of leaves **must be preserved**, as it directly affects the resulting Merkle root.<br />
    /// Returns an error if the input slice is empty.
    fn new(leaves: &[Hash]) -> Result<Self, AppError> {
        if leaves.is_empty() {
            return Err(AppError::internal(
                "Cannot create Merkle tree with no leaves!",
            ));
        }

        let mut current_layer = leaves.to_vec();

        // Pad the layer if it has an odd number of nodes.
        // Unless it's the final root node.
        if current_layer.len() % 2 != 0 {
            current_layer.push(current_layer.last().unwrap().clone());
        }

        let mut nodes = current_layer.clone();

        while current_layer.len() > 1 {
            let mut next_layer = Vec::with_capacity(current_layer.len().div_ceil(2));
            for i in (0..current_layer.len()).step_by(2) {
                let mut hasher = Blake2b512::new();
                hasher.update(current_layer[i].as_bytes());
                hasher.update(current_layer[i + 1].as_bytes());
                let digest = hasher.finalize();
                let mut hash_arr = [0u8; 32];
                hash_arr.copy_from_slice(&digest[..32]);
                next_layer.push(Hash::new(hash_arr));
            }

            // Pad if odd and not root
            if next_layer.len() % 2 != 0 && next_layer.len() != 1 {
                next_layer.push(next_layer.last().unwrap().clone());
            }

            nodes.extend_from_slice(&next_layer);
            current_layer = next_layer;
        }

        Ok(Self { nodes })
    }

    fn get_root(&self) -> Hash {
        self.nodes
            .last()
            .expect("MerkleTree invariant violated! No nodes present despite constructor checks.")
            .clone()
    }
}

impl std::fmt::Display for TransactionsMerkleRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
