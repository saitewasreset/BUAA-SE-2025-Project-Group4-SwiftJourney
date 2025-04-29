use crate::domain::Identifier;
use crate::domain::model::user::UserId;
use id_macro::define_id_type;
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

define_id_type!(Transaction);

pub struct Transaction {
    transaction_id: TransactionId,
    create_time: DateTimeWithTimeZone,
    finish_time: Option<DateTimeWithTimeZone>,
    amount: Decimal,
    status: TransactionStatus,
    user_id: UserId,
}
