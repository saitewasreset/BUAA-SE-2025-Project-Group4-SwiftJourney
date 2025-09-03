use crate::domain::model::order::{OrderStatus, OrderType};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Debug, Error)]
pub enum OrderStatusConsumerError {
    #[error("processing error: {0}")]
    ProcessingError(anyhow::Error),
    #[error("related service error: {0}")]
    RelatedServiceError(anyhow::Error),
}

#[async_trait]
pub trait RabbitMQOrderStatusConsumer: 'static + Send + Sync {
    fn binding_key(&self) -> &'static str;
    async fn consume(
        &self,
        message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError>;
}

#[async_trait]
pub trait OrderStatusConsumer: 'static + Send + Sync {
    async fn consume_order_status_change(
        &self,
        messages: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError>;
}

#[async_trait]
impl<T> OrderStatusConsumer for T
where
    T: RabbitMQOrderStatusConsumer,
{
    async fn consume_order_status_change(
        &self,
        order_status_message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError> {
        self.consume(order_status_message_pack).await
    }
}
