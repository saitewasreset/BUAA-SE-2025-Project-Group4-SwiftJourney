use crate::domain::model::order::{OrderStatus, TrainOrder};
use crate::domain::model::transaction::TransactionStatus;
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TrainBookingServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("no available tickets for order uuid: {0}")]
    NoAvailableTickets(Uuid),
    #[error("no order found for order uuid: {0}")]
    InvalidOrder(Uuid),
    #[error("invalid order status for order uuid: {0}, status: {1}")]
    InvalidOrderStatus(Uuid, OrderStatus),
    #[error("no transaction found for transaction uuid: {0}")]
    InvalidTransaction(Uuid),
    #[error("invalid transaction status for transaction uuid: {0}, status: {1}")]
    InvalidTransactionStatus(Uuid, TransactionStatus),
}

#[async_trait]
pub trait TrainBookingService: 'static + Send + Sync {
    async fn booking_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError>;
    async fn cancel_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError>;

    // 返回要退款的订单
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<TrainOrder>, TrainBookingServiceError>;
}
