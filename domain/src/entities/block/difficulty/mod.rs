use bincode::{Decode, Encode};

#[derive(Clone, Debug, Encode, Decode)]
pub struct BlockDifficultyTarget(pub(super) u128);

impl BlockDifficultyTarget {
    pub fn as_u128(&self) -> u128 {
        self.0
    }

    /// **Unimplemented Stub**
    pub fn _new_stub() -> Self {
        Self(0)
    }
}
