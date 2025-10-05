use super::MasterKeyAuthContext;
use common::error::AppError;

pub trait MasterKeyAuthenticator: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn authenticate_master_key(&self, secret: String) -> Result<MasterKeyAuthContext, AppError>;
}
