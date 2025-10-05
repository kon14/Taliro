use bincode::{Decode, Encode};

#[derive(Clone, Debug, Encode, Decode, Default)]
pub struct BlockNonce(pub(super) u64);

impl BlockNonce {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
