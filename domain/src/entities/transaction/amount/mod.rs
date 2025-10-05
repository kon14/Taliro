use bincode::{Decode, Encode};
use common::error::AppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Encode, Decode)]
pub struct TransactionAmount(pub(super) u128);

impl TransactionAmount {
    pub fn new(amount: u128) -> Self {
        Self(amount)
    }

    pub fn as_u128(&self) -> u128 {
        self.0
    }

    pub fn checked_add(&self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_add_assign(&mut self, other: Self) -> Result<(), AppError> {
        self.0 = self
            .0
            .checked_add(other.0)
            .ok_or(AppError::internal(format!(
                "Overflow in TransactionAmount: {} + {}",
                self.0, other.0
            )))?;
        Ok(())
    }

    pub fn checked_sub(&self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    pub fn checked_sub_assign(&mut self, other: Self) -> Result<(), AppError> {
        self.0 = self
            .0
            .checked_sub(other.0)
            .ok_or(AppError::internal(format!(
                "Underflow in TransactionAmount: {} - {}",
                self.0, other.0
            )))?;
        Ok(())
    }
}
