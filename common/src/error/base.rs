#[derive(Debug)]
pub struct BaseError {
    /// Public information that's safe to expose to end-users.
    public_info: String,

    /// Private information intended for internal logging and debugging.
    private_info: Option<String>,
}

impl std::fmt::Display for BaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.public_info)
    }
}

impl BaseError {
    pub(super) fn new(public_info: String, private_info: Option<String>) -> Self {
        Self {
            public_info,
            private_info,
        }
    }

    #[allow(unused)]
    pub(super) fn get_public_info(&self) -> &str {
        &self.public_info
    }

    #[allow(unused)]
    pub(super) fn get_private_info(&self) -> Option<&str> {
        self.private_info.as_deref()
    }

    pub fn reword<S>(self, public_info: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            public_info: public_info.into(),
            private_info: self.private_info,
        }
    }
}
