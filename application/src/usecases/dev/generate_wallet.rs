use common::error::AppError;
use domain::types::sign::{PrivateKey, PublicKey};
use domain::types::wallet::WalletAddress;

// TODO: Support mnemonic seed generation.

#[derive(Clone)]
pub struct GenerateWalletUseCase;

impl GenerateWalletUseCase {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self) -> Result<GenerateWalletUseCaseResponse, AppError> {
        let private_key = PrivateKey::generate();
        let public_key = private_key.get_public_key();
        let wallet_address = (&public_key).into();

        let res = GenerateWalletUseCaseResponse {
            private_key,
            public_key,
            wallet_address,
        };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct GenerateWalletUseCaseResponse {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub wallet_address: WalletAddress,
}
