use crate::domain::service::dish_booking::{DishBookingService, DishBookingServiceError};
use async_trait::async_trait;
use shared::domain::ServiceError;
use shared::domain::model::order::{DishOrder, Order, OrderStatus, OrderType};
use shared::internal::order::command::{OrderByUuidQuery, UpdateOrdersCommand};
use shared::ports::order::OrderPort;
use std::any::Any;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct DishBookingServiceImpl<OP>
where
    OP: OrderPort,
{
    order_port: Arc<OP>,
}

impl<OP> DishBookingServiceImpl<OP>
where
    OP: OrderPort,
{
    pub fn new(order_port: Arc<OP>) -> Self {
        Self { order_port }
    }
}

#[async_trait]
impl<OP> DishBookingService for DishBookingServiceImpl<OP>
where
    OP: OrderPort,
{
    #[instrument(skip(self))]
    async fn booking_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError> {
        let mut order = match self
            .order_port
            .get_order_by_uuid(OrderByUuidQuery { order_uuid })
            .await
        {
            Ok(Some(order)) => {
                let order_dyn: Box<dyn Order> = order.into();

                if order_dyn.order_type() == OrderType::Dish {
                    *(order_dyn as Box<dyn Any>).downcast::<DishOrder>().unwrap()
                } else {
                    error!(
                        "Dish order type is not dish: {:?} order uuid = {}",
                        order_dyn.order_type(),
                        order_dyn.uuid()
                    );

                    return Err(DishBookingServiceError::InvalidOrder(order_dyn.uuid()));
                }
            }
            Ok(None) => {
                error!("Order {} not found", order_uuid);

                return Err(DishBookingServiceError::InvalidOrder(order_uuid));
            }
            Err(e) => {
                error!("Error finding order {}: {:?}", order_uuid, e);

                return Err(DishBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(e.into()),
                ));
            }
        };

        if order.order_status() != OrderStatus::Paid {
            return Err(DishBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        // 火车餐订单总是会成功

        order.set_status(OrderStatus::Ongoing);

        self.order_port
            .update_orders(UpdateOrdersCommand {
                orders: vec![(Box::new(order) as Box<dyn Order>).as_ref().into()],
            })
            .await
            .inspect_err(|e| {
                error!("Failed to update order status: {}", e);
            })
            .map_err(|e| {
                DishBookingServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    e.into(),
                ))
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError> {
        let mut order = match self
            .order_port
            .get_order_by_uuid(OrderByUuidQuery { order_uuid })
            .await
        {
            Ok(Some(order)) => {
                let order_dyn: Box<dyn Order> = order.into();

                if order_dyn.order_type() == OrderType::Dish {
                    *(order_dyn as Box<dyn Any>).downcast::<DishOrder>().unwrap()
                } else {
                    error!(
                        "Dish order type is not dish: {:?} order uuid = {}",
                        order_dyn.order_type(),
                        order_dyn.uuid()
                    );

                    return Err(DishBookingServiceError::InvalidOrder(order_dyn.uuid()));
                }
            }
            Ok(None) => {
                error!("Order {} not found", order_uuid);

                return Err(DishBookingServiceError::InvalidOrder(order_uuid));
            }
            Err(e) => {
                error!("Error finding order {}: {:?}", order_uuid, e);

                return Err(DishBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(e.into()),
                ));
            }
        };

        if order.order_status() != OrderStatus::Ongoing {
            return Err(DishBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        order.set_status(OrderStatus::Cancelled);

        self.order_port
            .update_orders(UpdateOrdersCommand {
                orders: vec![(Box::new(order) as Box<dyn Order>).as_ref().into()],
            })
            .await
            .inspect_err(|e| {
                error!("Failed to update order status: {}", e);
            })
            .map_err(|e| {
                DishBookingServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    e.into(),
                ))
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
