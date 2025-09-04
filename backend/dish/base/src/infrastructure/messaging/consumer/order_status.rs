use crate::domain::service::dish_booking::DishBookingService;
use crate::domain::service::takeaway_booking::TakeawayBookingService;
use async_trait::async_trait;
use shared::{
    domain::model::order::{Order, OrderStatus, OrderType},
    internal::order::{command::RefundTransactionCommand, dto::InternalOrderDTO},
    messaging::order_status::{
        OrderStatusConsumerError, OrderStatusMessagePack, RabbitMQOrderStatusConsumer,
    },
    ports::order::OrderPort,
};
use std::sync::Arc;
use tracing::{error, info, instrument};

pub struct DishOrderStatusConsumer<DBS, OP>
where
    DBS: DishBookingService,
    OP: OrderPort,
{
    dish_booking_service: Arc<DBS>,
    order_port: Arc<OP>,
}

impl<DBS, OP> DishOrderStatusConsumer<DBS, OP>
where
    DBS: DishBookingService,
    OP: OrderPort,
{
    pub fn new(dish_booking_service: Arc<DBS>, order_port: Arc<OP>) -> Self {
        Self {
            dish_booking_service,
            order_port,
        }
    }
}

pub struct TakeawayOrderStatusConsumer<TBS, OP>
where
    TBS: TakeawayBookingService,
    OP: OrderPort,
{
    takeaway_booking_service: Arc<TBS>,
    order_port: Arc<OP>,
}

impl<TBS, OP> TakeawayOrderStatusConsumer<TBS, OP>
where
    TBS: TakeawayBookingService,
    OP: OrderPort,
{
    pub fn new(takeaway_booking_service: Arc<TBS>, order_port: Arc<OP>) -> Self {
        Self {
            takeaway_booking_service,
            order_port,
        }
    }
}

#[async_trait]
impl<DBS, OP> RabbitMQOrderStatusConsumer for DishOrderStatusConsumer<DBS, OP>
where
    DBS: DishBookingService,
    OP: OrderPort,
{
    fn binding_key(&self) -> &'static str {
        OrderType::Dish.message_queue_name()
    }

    #[instrument(skip(self))]
    async fn consume(
        &self,
        message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError> {
        info!("Processing Dish order status change");

        let mut to_cancel_order_id_list = Vec::new();
        let mut to_booking_order_id_list = Vec::new();

        for message in message_pack.messages {
            if message.order_type != OrderType::Dish {
                error!(
                    "invalid order type for dish order consumer: {}",
                    message.order_type
                );
            }

            match message.new_status {
                OrderStatus::Paid => to_booking_order_id_list.push(message.order_id),
                OrderStatus::Cancelled => to_cancel_order_id_list.push(message.order_id),
                x => {
                    error!("unexpected order status: {}", x);
                }
            }
        }

        let tx = self
            .dish_booking_service
            .booking_group(to_booking_order_id_list, message_pack.atomic)
            .await
            .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;

        if !tx.is_empty() {
            let tx_list_dto: Vec<InternalOrderDTO> = tx
                .into_iter()
                .map(|tx| (Box::new(tx) as Box<dyn Order>).as_ref().into())
                .collect::<Vec<_>>();

            self.order_port
                .refund_transaction(RefundTransactionCommand {
                    transaction_id: message_pack.transaction_uuid,
                    to_refund_orders: tx_list_dto,
                })
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        for order_uuid in to_cancel_order_id_list {
            self.dish_booking_service
                .cancel_dish(order_uuid)
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        Ok(())
    }
}

#[async_trait]
impl<TBS, OP> RabbitMQOrderStatusConsumer for TakeawayOrderStatusConsumer<TBS, OP>
where
    TBS: TakeawayBookingService,
    OP: OrderPort,
{
    fn binding_key(&self) -> &'static str {
        OrderType::Takeaway.message_queue_name()
    }

    #[instrument(skip(self))]
    async fn consume(
        &self,
        message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError> {
        info!("Processing Takeaway order status change");

        let mut to_cancel_order_id_list = Vec::new();
        let mut to_booking_order_id_list = Vec::new();

        for message in message_pack.messages {
            if message.order_type != OrderType::Takeaway {
                error!(
                    "invalid order type for takeaway consumer: {}",
                    message.order_type
                );
            }

            match message.new_status {
                OrderStatus::Paid => to_booking_order_id_list.push(message.order_id),
                OrderStatus::Cancelled => to_cancel_order_id_list.push(message.order_id),
                x => {
                    error!("unexpected order status: {}", x);
                }
            }
        }

        let tx = self
            .takeaway_booking_service
            .booking_group(to_booking_order_id_list, message_pack.atomic)
            .await
            .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;

        if !tx.is_empty() {
            let tx_list_dto: Vec<InternalOrderDTO> = tx
                .into_iter()
                .map(|tx| (Box::new(tx) as Box<dyn Order>).as_ref().into())
                .collect::<Vec<_>>();

            self.order_port
                .refund_transaction(RefundTransactionCommand {
                    transaction_id: message_pack.transaction_uuid,
                    to_refund_orders: tx_list_dto,
                })
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        for order_uuid in to_cancel_order_id_list {
            self.takeaway_booking_service
                .cancel_takeaway(order_uuid)
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        Ok(())
    }
}
