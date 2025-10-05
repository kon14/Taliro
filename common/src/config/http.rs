use crate::error::AppError;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PartialHttpConfig {
    pub api_port: Option<u16>,
    pub api_base_url: Option<String>,
    pub master_key_secret: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    pub api_port: u16,
    pub api_base_url: String,
    pub master_key_secret: Option<String>,
}

impl HttpConfig {
    const DEFAULT_API_PORT: u16 = 4000;
    const DEFAULT_API_BASE_URL: &'static str = "http://localhost:4000";

    pub(super) fn from_parts(
        base: PartialHttpConfig,
        overrides: PartialHttpConfig,
    ) -> Result<Self, AppError> {
        let api_port = overrides
            .api_port
            .or(base.api_port)
            .unwrap_or(Self::DEFAULT_API_PORT);

        let api_base_url = overrides
            .api_base_url
            .or(base.api_base_url)
            .unwrap_or(Self::DEFAULT_API_BASE_URL.to_string());

        let master_key_secret = overrides.master_key_secret.or(base.master_key_secret);

        let config = HttpConfig {
            api_port,
            api_base_url,
            master_key_secret,
        };
        Ok(config)
    }
}
