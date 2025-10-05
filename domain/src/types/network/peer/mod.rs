use bincode::{Decode, Encode};
use std::fmt;

/// An opaque wrapper over the serialized network peer identifier.
#[derive(Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct NetworkPeerId(Vec<u8>, String);

impl NetworkPeerId {
    pub fn new_unchecked(bytes: Vec<u8>, str_repr: String) -> Self {
        Self(bytes, str_repr)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        &self.1
    }
}

impl fmt::Display for NetworkPeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.1)
    }
}

impl fmt::Debug for NetworkPeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetworkPeerId({})", self)
    }
}
