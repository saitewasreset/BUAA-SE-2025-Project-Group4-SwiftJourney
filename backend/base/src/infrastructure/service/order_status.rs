use crate::ORDER_STATUS_UPDATE_INTERVAL_SECONDS;
use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::order_status::OrderStatusManagerService;
use crate::domain::service::order_status::{OrderStatusMessage, OrderStatusMessagePack};
use crate::infrastructure::service::order_status_producer_service::OrderStatusProducerService;
use async_trait::async_trait;
use chrono::Local;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

pub struct OrderStatusManagerServiceImpl<OR>
where
    OR: OrderRepository,
{
    order_status_producer_service: Arc<OrderStatusProducerService>,
    order_repository: Arc<OR>,
}

impl<OR> OrderStatusManagerServiceImpl<OR>
where
    OR: OrderRepository,
{
    pub fn new(
        order_status_producer_service: Arc<OrderStatusProducerService>,
        order_repository: Arc<OR>,
    ) -> Self {
        Self {
            order_status_producer_service,
            order_repository,
        }
    }

    #[instrument(skip(self))]
    async fn update_order_status(&self) -> Result<(), anyhow::Error> {
        info!("Updating order status...");

        let active_orders = self
            .order_repository
            .load_all_active_orders()
            .await
            .inspect_err(|e| error!("Failed to load active orders: {}", e))?;

        info!("Found {} active orders", active_orders.len());

        let mut to_update_orders = Vec::new();

        let now = Local::now();

        for mut order in active_orders {
            let prev_status = order.order_status();

            if now >= order.order_time_info().active_time()
                && now < order.order_time_info().complete_time()
            {
                order.set_status(OrderStatus::Ongoing)
            } else if now >= order.order_time_info().complete_time() {
                order.set_status(OrderStatus::Completed);
            } else {
                continue; // Skip orders that are not in the active or completed state
            }

            if prev_status != order.order_status() {
                to_update_orders.push(order);
            }
        }

        info!("{} orders need status update", to_update_orders.len());

        for order in to_update_orders {
            if let Err(e) = self.order_repository.update(order).await {
                error!("Failed to update order status: {}", e);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<OR> OrderStatusManagerService for OrderStatusManagerServiceImpl<OR>
where
    OR: OrderRepository,
{
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

    #[instrument(skip_all)]
    async fn order_status_daemon(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
            ORDER_STATUS_UPDATE_INTERVAL_SECONDS,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.update_order_status().await {
                error!("Failed to update order status: {}", e);
            }
        }
    }
}
