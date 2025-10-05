use common::error::{AppError, CryptographicError, StorageError};
use libp2p::TransportError;
use sled::transaction::{
    ConflictableTransactionError, TransactionError, UnabortableTransactionError,
};
use tokio::sync::mpsc::error::SendError;

pub(crate) trait AppErrorExtInfrastructure {
    type Type;

    fn to_app_error(self) -> Result<Self::Type, AppError>;
}

impl<T> AppErrorExtInfrastructure for Result<T, sled::Error> {
    type Type = T;

    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::Storage(StorageError::GenericError {
                reason: err.to_string(),
            })),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, UnabortableTransactionError> {
    type Type = T;

    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::Storage(StorageError::TransactionFailed {
                reason: err.to_string(),
            })),
        }
    }
}

impl<T> AppErrorExtInfrastructure
    for Result<T, TransactionError<ConflictableTransactionError<AppError>>>
{
    type Type = T;

    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(val) => Ok(val),
            Err(TransactionError::Abort(ConflictableTransactionError::Abort(err))) => {
                Err(AppError::Storage(StorageError::TransactionFailed {
                    reason: err.to_string(),
                }))
            }
            Err(TransactionError::Abort(ConflictableTransactionError::Conflict)) => {
                Err(AppError::Storage(StorageError::TransactionFailed {
                    reason: "Conflict".to_string(),
                }))
            }
            Err(TransactionError::Storage(err)) => {
                Err(AppError::Storage(StorageError::TransactionFailed {
                    reason: err.to_string(),
                }))
            }
            Err(other) => Err(AppError::Storage(StorageError::TransactionFailed {
                reason: other.to_string(),
            })),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, ConflictableTransactionError> {
    type Type = T;

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::Storage(StorageError::WriteFailed {
                reason: err.to_string(),
            })),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, TransactionError<AppError>> {
    type Type = T;

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::Storage(StorageError::TransactionFailed {
                reason: err.to_string(),
            })),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, libp2p::gossipsub::SubscriptionError> {
    type Type = ();

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(_) => Ok(()),
            // TODO: remap
            Err(err) => Err(AppError::internal(format!(
                "libp2p Gossipsub subscription error: {}",
                err
            ))),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, libp2p::noise::Error> {
    type Type = T;

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(value) => Ok(value),
            // TODO: remap
            Err(err) => Err(AppError::internal(format!("libp2p Noise error: {}", err))),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, std::io::Error> {
    type Type = T;

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(value) => Ok(value),
            // TODO: remap
            Err(err) => Err(AppError::internal(format!("std::io error: {}", err))),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, libp2p::BehaviourBuilderError> {
    type Type = T;

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(value) => Ok(value),
            // TODO: remap
            Err(err) => Err(AppError::internal(format!(
                "libp2p BehaviourBuilder error: {}",
                err
            ))),
        }
    }
}

impl<T, TErr> AppErrorExtInfrastructure for Result<T, TransportError<TErr>> {
    type Type = T;

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(value) => Ok(value),
            // TODO: remap
            Err(_) => Err(AppError::internal(format!("libp2p Transport error!"))),
        }
    }
}

impl<TErr> AppErrorExtInfrastructure for Result<(), SendError<TErr>> {
    type Type = ();

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        match self {
            Ok(value) => Ok(value),
            // TODO: remap
            Err(err) => Err(AppError::internal(format!("tokio send error: {}", err))),
        }
    }
}

impl<T> AppErrorExtInfrastructure for Result<T, libp2p::identity::DecodingError> {
    type Type = T;

    fn to_app_error(self) -> Result<Self::Type, AppError> {
        self.map_err(|err| {
            AppError::Cryptographic(CryptographicError::DecodingFailed {
                reason: err.to_string(),
            })
        })
    }
}
