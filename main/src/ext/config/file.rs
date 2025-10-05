use super::PartialAppConfigFromFileExtMain;
use common::config::PartialAppConfig;
use common::error::AppError;
use std::path::PathBuf;
use std::{env, fs};

impl PartialAppConfigFromFileExtMain for PartialAppConfig {
    fn load_from_file() -> Result<Self, AppError> {
        const ERR_MSG_PREFIX: &str = "Failed to read application config file:";

        let path = get_env_config_file_path()?.or_else(get_default_config_file_path);
        let path = if let Some(path) = path {
            path
        } else {
            return Ok(Self::default());
        };

        let contents = fs::read_to_string(&path).map_err(|err| {
            AppError::internal(format!(
                "{} {} | Error: {}",
                ERR_MSG_PREFIX,
                &path.to_string_lossy(),
                err
            ))
        })?;
        let config: Self = toml::from_str(&contents).map_err(|err| {
            AppError::internal(format!(
                "{} {} | Error: {}",
                ERR_MSG_PREFIX,
                &path.to_string_lossy(),
                err
            ))
        })?;
        Ok(config)
    }
}

/// Retrieves the env-specified config file path.<br />
/// Returns an error for invalid paths.
fn get_env_config_file_path() -> Result<Option<PathBuf>, AppError> {
    const CFG_PATH_ENV: &'static str = "CONFIG_PATH";
    const ERR_MSG_PREFIX: &str = "Failed to read application config file:";

    let env_path = env::var(CFG_PATH_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from);
    if let Some(path) = env_path {
        if path.exists() && !path.is_file() {
            let path_str = path.to_string_lossy();
            return Err(AppError::internal(format!(
                "{} {} | Error: File doesn't exist or isn't a file!",
                ERR_MSG_PREFIX, path_str
            )));
        }
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

/// Retrieves the default config file path.
fn get_default_config_file_path() -> Option<PathBuf> {
    const DEFAULT_CFG_PATH: &'static str = "/config.toml";

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut default_path = PathBuf::from(manifest_dir)
        .parent()
        .map(|p| p.to_path_buf())?;
    default_path.push(DEFAULT_CFG_PATH.trim_start_matches('/'));

    if default_path.exists() && !default_path.is_file() {
        Some(default_path)
    } else {
        None
    }
}
