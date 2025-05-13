use crate::domain::model::order::{OrderStatus, OrderType};
use crate::domain::service::order_status_consumer::{
    OrderStatusConsumer, OrderStatusConsumerError,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusMessage {
    order_id: Uuid,
    order_type: OrderType,
    new_status: OrderStatus,
}

#[async_trait]
pub trait RabbitMQOrderStatusConsumer: 'static + Send + Sync {
    fn channel(&self) -> &lapin::Channel;
    fn binding_key(&self) -> &str;
    async fn consume(&self, message: OrderStatusMessage) -> Result<(), OrderStatusConsumerError>;
}

#[async_trait]
impl<T> OrderStatusConsumer for T
where
    T: RabbitMQOrderStatusConsumer,
{
    async fn consume_order_status_change(
        &self,
        order_id: Uuid,
        order_type: OrderType,
        new_status: OrderStatus,
    ) -> Result<(), OrderStatusConsumerError> {
        self.consume(OrderStatusMessage {
            order_id,
            order_type,
            new_status,
        })
        .await
    }
}
