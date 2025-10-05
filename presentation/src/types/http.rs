use std::{fmt, ops::Deref};

pub struct ServerAddress(pub(crate) String);

impl fmt::Display for ServerAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ServerAddress {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
