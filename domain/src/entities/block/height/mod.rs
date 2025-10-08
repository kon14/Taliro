use bincode::{Decode, Encode};
use std::fmt;

#[derive(Clone, Encode, Decode, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockHeight(pub(super) u64);

impl BlockHeight {
    const GENESIS_BLOCK_HEIGHT: u64 = u64::MIN;

    pub fn to_be_bytes(&self) -> [u8; size_of::<u64>()] {
        self.0.to_be_bytes()
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn genesis() -> Self {
        Self(Self::GENESIS_BLOCK_HEIGHT)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u64> for BlockHeight {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Debug for BlockHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockHeight({})", self.0)
    }
}

impl fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
