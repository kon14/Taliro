use crate::error::AppError;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PartialNodeConfig {}

#[derive(Clone, Debug, Deserialize)]
pub struct NodeConfig {}

impl NodeConfig {
    #[allow(unused)]
    pub(super) fn from_parts(
        base: PartialNodeConfig,
        overrides: PartialNodeConfig,
    ) -> Result<Self, AppError> {
        let config = NodeConfig {};
        Ok(config)
    }
}
