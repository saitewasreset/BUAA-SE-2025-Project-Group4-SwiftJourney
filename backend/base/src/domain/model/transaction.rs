use crate::domain::Identifier;
use crate::domain::model::user::UserId;
use sea_orm::prelude::{DateTimeWithTimeZone, Decimal};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransactionStatus {
    Unpaid,
    Paid,
}

#[derive(Error, Debug)]
pub enum TransactionStatusError {
    #[error("Invalid transaction status: {0}")]
    InvalidStatus(String),
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Unpaid => write!(f, "Unpaid"),
            TransactionStatus::Paid => write!(f, "Paid"),
        }
    }
}

impl From<TransactionStatus> for &'static str {
    fn from(status: TransactionStatus) -> Self {
        match status {
            TransactionStatus::Unpaid => "Unpaid",
            TransactionStatus::Paid => "Paid",
        }
    }
}

impl TryFrom<&str> for TransactionStatus {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Unpaid" => Ok(TransactionStatus::Unpaid),
            "Paid" => Ok(TransactionStatus::Paid),
            _ => Err("Invalid transaction status"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(u64);

#[derive(Error, Debug)]
pub enum TransactionIdError {
    #[error("Invalid negative value for transaction id")]
    NegativeValue,
}

impl TryFrom<i32> for TransactionId {
    type Error = TransactionIdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            Err(TransactionIdError::NegativeValue)
        } else {
            Ok(TransactionId(value as u64))
        }
    }
}

impl From<u64> for TransactionId {
    fn from(value: u64) -> Self {
        TransactionId(value)
    }
}

impl From<TransactionId> for u64 {
    fn from(transaction_id: TransactionId) -> Self {
        transaction_id.0
    }
}

impl Identifier for TransactionId {}

pub struct Transaction {
    transaction_id: TransactionId,
    create_time: DateTimeWithTimeZone,
    finish_time: Option<DateTimeWithTimeZone>,
    amount: Decimal,
    status: TransactionStatus,
    user_id: UserId,
}
