use crate::domain::model::order::{OrderStatus, OrderType};
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[async_trait]
pub trait OrderStatusConsumer: 'static + Send + Sync {
    async fn consume_order_status_change(
        &self,
        order_id: Uuid,
        order_type: OrderType,
        new_status: OrderStatus,
    ) -> Result<(), OrderStatusConsumerError>;
}

#[derive(Debug, Error)]
pub enum OrderStatusConsumerError {
    #[error("processing error: {0}")]
    ProcessingError(anyhow::Error),
    #[error("related service error: {0}")]
    RelatedServiceError(anyhow::Error),
}
