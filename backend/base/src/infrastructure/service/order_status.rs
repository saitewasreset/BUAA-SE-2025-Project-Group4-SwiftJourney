use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::service::order_status::OrderStatusManagerService;
use crate::domain::service::order_status::{OrderStatusMessage, OrderStatusMessagePack};
use crate::infrastructure::service::order_status_producer_service::OrderStatusProducerService;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

pub struct OrderStatusManagerServiceImpl {
    order_status_producer_service: Arc<OrderStatusProducerService>,
}

impl OrderStatusManagerServiceImpl {
    pub fn new(order_status_producer_service: Arc<OrderStatusProducerService>) -> Self {
        Self {
            order_status_producer_service,
        }
    }
}

#[async_trait]
impl OrderStatusManagerService for OrderStatusManagerServiceImpl {
    #[instrument(skip_all)]
    async fn notify_status_change(
        &self,
        transaction_uuid: Uuid,
        atomic: bool,
        orders: &[&dyn Order],
        new_status: OrderStatus,
    ) {
        info!(
            "order status change: transaction_uuid: {}, atomic: {}, new_status: {}",
            transaction_uuid, atomic, new_status
        );

        let mut messages = Vec::new();

        for order in orders {
            messages.push(OrderStatusMessage {
                order_id: order.uuid(),
                order_type: order.order_type(),
                new_status,
            });
        }

        let message_pack = OrderStatusMessagePack {
            transaction_uuid,
            atomic,
            messages,
        };

        if let Err(e) = self
            .order_status_producer_service
            .delivery_message(message_pack)
            .await
        {
            error!("error while producing order status change: {}", e);
        }
    }
}
