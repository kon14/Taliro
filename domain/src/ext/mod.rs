use bincode::error::{DecodeError, EncodeError};
use common::error::AppError;
use common::error::CryptographicError;

pub(crate) trait AppErrorConvertibleDomain {
    type Type;
    fn to_app_error(self) -> Result<Self::Type, AppError>;
}

impl<T> AppErrorConvertibleDomain for Result<T, EncodeError> {
    type Type = T;
    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::Cryptographic(
                CryptographicError::EncodingFailed {
                    reason: format!("Encoding error: {}", err),
                },
            )),
        }
    }
}

impl<T> AppErrorConvertibleDomain for Result<T, DecodeError> {
    type Type = T;
    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::Cryptographic(
                CryptographicError::DecodingFailed {
                    reason: format!("Decoding error: {}", err),
                },
            )),
        }
    }
}
