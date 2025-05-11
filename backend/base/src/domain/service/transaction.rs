//! # 交易领域服务模块
//!
//! 该模块定义了火车票订购系统中的交易领域服务接口。主要包含以下内容：
//!
//! - `TransactionServiceError`: 枚举类型，表示交易领域服务错误。
//! - `TransactionService`: 异步 trait，定义了交易领域的操作。
use crate::domain::RepositoryError;
use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::model::transaction::{
    RefundError, Transaction, TransactionAmountAbs, TransactionStatus,
};
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use crate::domain::service::order::order_dto::TransactionDataDto;
use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

/// 枚举类型，表示交易领域服务错误。
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

/// 异步 trait，定义了交易领域的操作。
///
/// 包含以下方法：
/// - `recharge`: 为用户充值。
/// - `get_balance`: 获取用户的余额。
/// - `new_transaction`: 创建新的交易。
/// - `pay_transaction`: 支付交易。
/// - `refund_transaction`: 退款交易。
#[async_trait]
pub trait TransactionService: 'static + Send + Sync {
    /// 为用户充值。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    /// - `amount`: 充值金额的绝对值。
    ///
    /// Returns:
    /// - 成功时返回充值交易的 UUID。
    /// - 失败时返回 `TransactionServiceError`。
    async fn recharge(
        &self,
        user_id: UserId,
        amount: TransactionAmountAbs,
    ) -> Result<Uuid, TransactionServiceError>;

    /// 获取用户的余额。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    ///
    /// Returns:
    /// - 成功时返回用户的余额。
    /// - 失败时返回 `TransactionServiceError`。
    async fn get_balance(&self, user_id: UserId) -> Result<Decimal, TransactionServiceError>;

    /// 创建新的交易。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    /// - `orders`: 交易包含的订单列表。
    ///
    /// Returns:
    /// - 成功时返回新创建交易的 UUID。
    /// - 失败时返回 `TransactionServiceError`。c
    async fn new_transaction(
        &self,
        user_id: UserId,
        orders: Vec<Box<dyn Order>>,
    ) -> Result<Uuid, TransactionServiceError>;

    /// 支付交易。
    ///
    /// Arguments:
    /// - `transaction_id`: 交易的 UUID。
    ///
    /// Returns:
    /// - 成功时返回 `Ok(())`。
    /// - 失败时返回 `TransactionServiceError`。
    async fn pay_transaction(&self, transaction_id: Uuid) -> Result<(), TransactionServiceError>;

    /// 退款交易。
    ///
    /// Arguments:
    /// - `transaction_id`: 交易的 UUID。
    /// - `to_refund_orders`: 要退款的订单列表。
    ///
    /// Returns:
    /// - 成功时返回新的退款交易的 UUID。
    /// - 失败时返回 `TransactionServiceError`。
    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
        to_refund_orders: &[Box<dyn Order>],
    ) -> Result<Uuid, TransactionServiceError>;

    async fn convert_transaction_to_dto(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionDataDto, TransactionServiceError>;
}
