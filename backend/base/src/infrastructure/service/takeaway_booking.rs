use crate::domain::model::order::{Order, OrderStatus, TakeawayOrder};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::takeaway_booking::{
    TakeawayBookingService, TakeawayBookingServiceError,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct TakeawayBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    order_repository: Arc<OR>,
}

impl<OR> TakeawayBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    pub fn new(order_repository: Arc<OR>) -> Self {
        Self { order_repository }
    }
}

#[async_trait]
impl<OR> TakeawayBookingService for TakeawayBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    #[instrument(skip(self))]
    async fn booking_takeaway(&self, order_uuid: Uuid) -> Result<(), TakeawayBookingServiceError> {
        let mut order = self
            .order_repository
            .find_takeaway_order_by_uuid(order_uuid)
            .await?
            .ok_or(TakeawayBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Paid {
            return Err(TakeawayBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        // 外卖订单总是会成功

        order.set_status(OrderStatus::Ongoing);

        self.order_repository
            .update(Box::new(order))
            .await
            .inspect_err(|e| {
                error!("Failed to update order status: {}", e);
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_takeaway(&self, order_uuid: Uuid) -> Result<(), TakeawayBookingServiceError> {
        let mut order = self
            .order_repository
            .find_takeaway_order_by_uuid(order_uuid)
            .await?
            .ok_or(TakeawayBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Ongoing {
            return Err(TakeawayBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        order.set_status(OrderStatus::Cancelled);

        self.order_repository
            .update(Box::new(order))
            .await
            .inspect_err(|e| {
                error!("Failed to update order status: {}", e);
            })?;

        Ok(())
    }

    /// 对于合法的外卖订单，总是会成功，故本函数固定返回空的退款订单列表
    #[instrument(skip(self))]
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<TakeawayOrder>, TakeawayBookingServiceError> {
        let mut success_booking_order_list = Vec::new();

        for order_uuid in order_uuid_list {
            if let Err(e) = self.booking_takeaway(order_uuid).await {
                error!("Failed to book takeaway: {:?}", e);
                break;
            } else {
                success_booking_order_list.push(order_uuid);
            }
        }

        if atomic {
            for order_uuid in &success_booking_order_list {
                if let Err(e) = self.cancel_takeaway(*order_uuid).await {
                    error!("Failed to cancel takeaway: {:?}", e);

                    return Err(e);
                }
            }
        }

        Ok(Vec::new())
    }
}
