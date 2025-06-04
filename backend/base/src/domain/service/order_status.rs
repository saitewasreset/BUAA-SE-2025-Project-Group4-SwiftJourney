use crate::domain::model::order::{Order, OrderStatus, OrderType};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum OrderStatusManagerServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("order {0} status is invalid: {1}")]
    InvalidStatus(Uuid, OrderStatus),
}
#[async_trait]
pub trait OrderStatusManagerService: 'static + Send + Sync {
    async fn notify_status_change(
        &self,
        transaction_uuid: Uuid,
        atomic: bool,
        orders: &[&dyn Order],
        new_status: OrderStatus,
    );

    async fn order_status_daemon(&self);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusMessagePack {
    pub transaction_uuid: Uuid,
    pub messages: Vec<OrderStatusMessage>,
    pub atomic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusMessage {
    pub order_id: Uuid,
    pub order_type: OrderType,
    pub new_status: OrderStatus,
}

#[async_trait]
pub trait OrderStatusConsumer: 'static + Send + Sync {
    async fn consume_order_status_change(
        &self,
        messages: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError>;
}

#[derive(Debug, Error)]
pub enum OrderStatusConsumerError {
    #[error("processing error: {0}")]
    ProcessingError(anyhow::Error),
    #[error("related service error: {0}")]
    RelatedServiceError(anyhow::Error),
}
