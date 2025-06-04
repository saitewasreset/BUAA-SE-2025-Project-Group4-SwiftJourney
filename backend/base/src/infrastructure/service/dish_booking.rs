use crate::domain::model::order::{DishOrder, Order, OrderStatus};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::dish_booking::{DishBookingService, DishBookingServiceError};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct DishBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    order_repository: Arc<OR>,
}

impl<OR> DishBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    pub fn new(order_repository: Arc<OR>) -> Self {
        Self { order_repository }
    }
}

#[async_trait]
impl<OR> DishBookingService for DishBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    #[instrument(skip(self))]
    async fn booking_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError> {
        let mut order = self
            .order_repository
            .find_dish_order_by_uuid(order_uuid)
            .await?
            .ok_or(DishBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Paid {
            return Err(DishBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        // 火车餐订单总是会成功

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
    async fn cancel_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError> {
        let mut order = self
            .order_repository
            .find_dish_order_by_uuid(order_uuid)
            .await?
            .ok_or(DishBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Ongoing {
            return Err(DishBookingServiceError::InvalidOrderStatus(
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

    /// 对于合法的火车餐订单，总是会成功，故本函数固定返回空的退款订单列表
    #[instrument(skip(self))]
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        _atomic: bool,
    ) -> Result<Vec<DishOrder>, DishBookingServiceError> {
        let mut success_booking_order_list = Vec::new();

        for order_uuid in order_uuid_list {
            if let Err(e) = self.booking_dish(order_uuid).await {
                error!("Failed to book dish: {:?}", e);
                break;
            } else {
                success_booking_order_list.push(order_uuid);
            }
        }

        Ok(Vec::new())
    }
}
