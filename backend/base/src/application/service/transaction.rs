use crate::application::commands::transaction::{
    BalanceQuery, GenerateDebugTransactionCommand, PayTransactionCommand, RechargeCommand,
    SetPaymentPasswordCommand, TransactionDetailQuery, TransactionQuery,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::transaction::Transaction;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::transaction::TransactionServiceError;
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RechargeDTO {
    pub amount: f64,
    #[serde(rename = "externalPaymentId")]
    pub external_payment_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentConfirmationDTO {
    #[serde(rename = "userPassword")]
    pub user_password: Option<String>,
    #[serde(rename = "paymentPassword")]
    pub payment_password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentPasswordInfoDTO {
    #[serde(rename = "userPassword")]
    pub user_password: String,
    #[serde(rename = "paymentPassword")]
    pub payment_password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransactionGenerateDTO {
    pub amount: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BalanceInfoDTO {
    pub balance: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransactionInfoDTO {
    pub transaction_id: Uuid,
    pub amount: f64,
    pub status: String,
}

impl From<Transaction> for TransactionInfoDTO {
    fn from(value: Transaction) -> Self {
        TransactionInfoDTO {
            transaction_id: value.uuid(),
            amount: value.raw_amount().to_f64().unwrap(),
            status: value.status().to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum TransactionApplicationServiceError {
    #[error("wrong payment password")]
    WrongPaymentPassword,
    #[error("wrong user password")]
    WrongUserPassword,
    #[error("too many payment password attempts")]
    TooManyPaymentPasswordAttempts,
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("cannot refund this transaction: {0}")]
    RefundError(String),
    #[error("{0}")]
    InvalidTransactionStatus(String),
    #[error("invalid payment password format")]
    InvalidPaymentPasswordFormat,
}

impl From<TransactionServiceError> for Box<dyn ApplicationError> {
    fn from(value: TransactionServiceError) -> Self {
        match value {
            TransactionServiceError::InvalidUser(x) => {
                Box::new(GeneralError::BadRequest(format!("invalid user id: {}", x)))
            }
            TransactionServiceError::InvalidTransactionId(x) => {
                Box::new(GeneralError::BadRequest(format!("invalid user id: {}", x)))
            }
            e @ TransactionServiceError::InvalidTransactionStatus {
                op: _,
                status: _,
                transaction_id: _,
            } => Box::new(
                TransactionApplicationServiceError::InvalidTransactionStatus(e.to_string()),
            ),
            TransactionServiceError::InsufficientFunds {
                transaction_id: _,
                balance: _,
                amount: _,
            } => Box::new(TransactionApplicationServiceError::InsufficientFunds),
            e @ TransactionServiceError::RefundError(..) => Box::new(
                TransactionApplicationServiceError::RefundError(e.to_string()),
            ),
            _ => Box::new(GeneralError::InternalServerError),
        }
    }
}

impl ApplicationError for TransactionApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TransactionApplicationServiceError::WrongPaymentPassword => 11001,
            TransactionApplicationServiceError::WrongUserPassword => 11002,
            TransactionApplicationServiceError::TooManyPaymentPasswordAttempts => 11003,
            TransactionApplicationServiceError::InsufficientFunds => 11004,
            TransactionApplicationServiceError::RefundError(_) => 11005,
            TransactionApplicationServiceError::InvalidTransactionStatus(_) => 11006,
            TransactionApplicationServiceError::InvalidPaymentPasswordFormat => 11007,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait TransactionApplicationService: 'static + Send + Sync {
    async fn recharge(&self, command: RechargeCommand) -> Result<(), Box<dyn ApplicationError>>;

    async fn query_balance(
        &self,
        query: BalanceQuery,
    ) -> Result<BalanceInfoDTO, Box<dyn ApplicationError>>;

    async fn query_transactions(
        &self,
        query: TransactionQuery,
    ) -> Result<Vec<TransactionInfoDTO>, Box<dyn ApplicationError>>;

    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn pay_transaction(
        &self,
        command: PayTransactionCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn generate_debug_transaction(
        &self,
        command: GenerateDebugTransactionCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>>;

    async fn query_transaction_details(
        &self,
        query: TransactionDetailQuery,
    ) -> Result<Vec<TransactionDataDto>, Box<dyn ApplicationError>>;
}
