pub mod ctx;

use crate::error::AppError;
use ctx::AtomicTransactionContext;
use std::any::Any;

pub struct AtomicTransactionOutput {
    inner: Box<dyn Any + Send>,
}

impl AtomicTransactionOutput {
    pub fn new<T: Send + 'static>(value: T) -> Self {
        Self {
            inner: Box::new(value),
        }
    }

    pub fn extract<T: 'static>(self) -> Result<T, AppError> {
        self.inner
            .downcast::<T>()
            .map(|boxed| *boxed)
            .map_err(|_| AppError::internal("Failed to extract transaction result!"))
    }
}

pub trait UnitOfWork: Send + Sync {
    fn run_in_transaction(
        &self,
        f: Box<
            dyn for<'a> FnMut(
                    &'a mut dyn AtomicTransactionContext,
                ) -> Result<AtomicTransactionOutput, AppError>
                + Send,
        >,
    ) -> Result<AtomicTransactionOutput, AppError>;
}
