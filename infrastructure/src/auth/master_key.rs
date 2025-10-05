use application::auth::master_key::{MasterKeyAuthContext, MasterKeyAuthenticator};
use common::error::AppError;
use common::{log_auth_info, log_auth_warn};

pub struct DefaultMasterKeyAuthenticator {
    master_key: Option<String>,
}

impl DefaultMasterKeyAuthenticator {
    pub fn new(master_key: Option<String>) -> Self {
        if master_key.is_some() {
            log_auth_info!("Master key authentication is enabled.");
        } else {
            log_auth_warn!("Master key authentication is disabled.");
        }
        Self { master_key }
    }
}

impl MasterKeyAuthenticator for DefaultMasterKeyAuthenticator {
    fn is_enabled(&self) -> bool {
        self.master_key.is_some()
    }

    fn authenticate_master_key(&self, secret: String) -> Result<MasterKeyAuthContext, AppError> {
        const UNAUTHORIZED_ERR_STR: &str = "Failed to authenticate user!";

        match &self.master_key {
            Some(expected) if secret == *expected => Ok(MasterKeyAuthContext {}),
            _ => Err(AppError::unauthorized_with_private(
                UNAUTHORIZED_ERR_STR,
                "Invalid master key!",
            )),
        }
    }
}
