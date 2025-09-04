use crate::domain::service::transaction::TransactionServiceError;
use async_trait::async_trait;
use rust_decimal::Decimal;
use shared::application_error::ApplicationError;
use shared::domain::model::order::OrderStatus;
use shared::domain::model::transaction::{RefundError, TransactionAmountAbs, TransactionStatus};
use shared::domain::model::user::UserId;
use shared::internal::order::command::{
    NewTransactionCommand, OrderByUuidQuery, RefundTransactionCommand, UpdateOrdersCommand,
    UserOrderListQuery, VerifyTrainOrderQuery,
};
use shared::internal::order::dto::InternalOrderDTO;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum OrderInternalServiceError {
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
    #[error(transparent)]
    RelatedServiceError(#[from] anyhow::Error),
}

impl From<TransactionServiceError> for OrderInternalServiceError {
    fn from(value: TransactionServiceError) -> Self {
        match value {
            TransactionServiceError::InfrastructureError(value) => {
                Self::RelatedServiceError(value.into())
            }
            TransactionServiceError::InvalidUser(x) => Self::InvalidUser(x),
            TransactionServiceError::InvalidTransactionId(x) => Self::InvalidTransactionId(x),
            TransactionServiceError::InvalidTransactionStatus {
                op,
                status,
                transaction_id,
            } => Self::InvalidTransactionStatus {
                op,
                status,
                transaction_id,
            },
            TransactionServiceError::InvalidOrder {
                order_id,
                transaction_id,
            } => Self::InvalidOrder {
                order_id,
                transaction_id,
            },
            TransactionServiceError::InvalidOrderStatus {
                op,
                status,
                order_id,
                transaction_id,
            } => OrderInternalServiceError::InvalidOrderStatus {
                op,
                status,
                order_id,
                transaction_id,
            },
            TransactionServiceError::InsufficientFunds {
                transaction_id,
                balance,
                amount,
            } => OrderInternalServiceError::InsufficientFunds {
                transaction_id,
                balance,
                amount,
            },
            TransactionServiceError::RefundError(x) => OrderInternalServiceError::RefundError(x),
        }
    }
}

impl ApplicationError for OrderInternalServiceError {
    fn error_code(&self) -> u32 {
        match self {
            OrderInternalServiceError::InvalidUser(_) => 95001,
            OrderInternalServiceError::InvalidTransactionId(_) => 95002,
            OrderInternalServiceError::InvalidTransactionStatus { .. } => 95003,
            OrderInternalServiceError::InvalidOrder { .. } => 95004,
            OrderInternalServiceError::InvalidOrderStatus { .. } => 95005,
            OrderInternalServiceError::InsufficientFunds { .. } => 95006,
            OrderInternalServiceError::RefundError(_) => 95007,
            OrderInternalServiceError::RelatedServiceError(_) => 95008,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait OrderInternalService: 'static + Send + Sync {
    async fn new_transaction(
        &self,
        command: NewTransactionCommand,
    ) -> Result<Uuid, OrderInternalServiceError>;

    async fn refund_transaction(
        &self,
        command: RefundTransactionCommand,
    ) -> Result<Uuid, OrderInternalServiceError>;

    async fn get_order_by_uuid(
        &self,
        query: OrderByUuidQuery,
    ) -> Result<Option<InternalOrderDTO>, OrderInternalServiceError>;

    async fn verify_train_order(
        &self,
        query: VerifyTrainOrderQuery,
    ) -> Result<bool, OrderInternalServiceError>;

    async fn update_orders(
        &self,
        command: UpdateOrdersCommand,
    ) -> Result<(), OrderInternalServiceError>;

    async fn get_order_list_by_user_id(
        &self,
        query: UserOrderListQuery,
    ) -> Result<Vec<InternalOrderDTO>, OrderInternalServiceError>;
}
