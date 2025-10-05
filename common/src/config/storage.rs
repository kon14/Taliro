use crate::error::AppError;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PartialStorageConfig {
    pub db_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StorageConfig {
    pub db_path: String,
}

impl StorageConfig {
    const DEFAULT_DB_PATH: &'static str = "./data/db"; // project root relative

    pub(super) fn from_parts(
        base: PartialStorageConfig,
        overrides: PartialStorageConfig,
    ) -> Result<Self, AppError> {
        let db_path = overrides
            .db_path
            .or(base.db_path)
            .unwrap_or(Self::DEFAULT_DB_PATH.to_string());

        let config = StorageConfig { db_path };
        Ok(config)
    }
}
