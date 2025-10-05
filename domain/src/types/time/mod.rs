use bincode::{Decode, Encode};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Encode, Decode)]
pub struct DateTime(u64);

/// Unix timestamp in milliseconds.
impl DateTime {
    pub fn from_ms(ms: u64) -> Self {
        DateTime(ms)
    }

    pub fn to_ms(&self) -> u64 {
        self.0
    }

    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO) // let's be real...
            .as_millis() as u64;
        Self(millis)
    }
}
