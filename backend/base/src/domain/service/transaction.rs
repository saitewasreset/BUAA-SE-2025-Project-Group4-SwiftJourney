use crate::domain::RepositoryError;
use crate::domain::model::order::{Order, OrderId, OrderStatus};
use crate::domain::model::transaction::{
    RefundError, TransactionAmountAbs, TransactionError, TransactionId, TransactionStatus,
};
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum TransactionServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid user id: {0}")]
    InvalidUser(UserId),
    #[error("invalid transaction uuid: {0}")]
    InvalidTransactionId(Uuid),
    #[error("invalid transaction status {status} for op {op} for transaction {transaction_id}")]
    InvalidTransactionStatus {
        op: &'static str,
        status: TransactionStatus,
        transaction_id: Uuid,
    },
    #[error("cannot find order {order_id} in transaction {transaction_id}")]
    InvalidOrder {
        order_id: Uuid,
        transaction_id: Uuid,
    },
    #[error(
        "invalid order status {status} for op {op} for order {order_id} transaction: {transaction_id:?}"
    )]
    InvalidOrderStatus {
        op: &'static str,
        status: OrderStatus,
        order_id: Uuid,
        transaction_id: Option<Uuid>,
    },
    #[error(
        "insufficient funds to pay transaction {transaction_id} required: {amount} but only {balance} available"
    )]
    InsufficientFunds {
        transaction_id: Uuid,
        balance: Decimal,
        amount: TransactionAmountAbs,
    },
    #[error(transparent)]
    RefundError(#[from] RefundError),
}

impl From<RepositoryError> for TransactionServiceError {
    fn from(value: RepositoryError) -> Self {
        TransactionServiceError::InfrastructureError(ServiceError::RepositoryError(value))
    }
}

#[async_trait]
pub trait TransactionService {
    async fn recharge(
        &self,
        user_id: UserId,
        amount: TransactionAmountAbs,
    ) -> Result<Uuid, TransactionServiceError>;

    async fn get_balance(&self, user_id: UserId) -> Result<Decimal, TransactionServiceError>;

    async fn new_transaction(
        &self,
        user_id: UserId,
        orders: Vec<Box<dyn Order>>,
    ) -> Result<Uuid, TransactionServiceError>;

    async fn pay_transaction(&self, transaction_id: Uuid) -> Result<(), TransactionServiceError>;

    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
        to_refund_orders: &[Box<dyn Order>],
    ) -> Result<Uuid, TransactionServiceError>;
}
