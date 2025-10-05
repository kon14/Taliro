use common::error::AppError;

pub trait TryEncode {
    fn try_encode(&self) -> Result<Vec<u8>, AppError>;
}

pub trait TryDecode {
    fn try_decode(data: &[u8]) -> Result<Self, AppError>
    where
        Self: Sized;
}
