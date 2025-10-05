use common::error::AppError;

mod env;
mod file;

pub(crate) trait PartialAppConfigFromFileExtMain {
    fn load_from_file() -> Result<Self, AppError>
    where
        Self: Sized;
}

pub(crate) trait PartialAppConfigFromEnvExtMain {
    fn load_from_env() -> Result<Self, AppError>
    where
        Self: Sized;
}
