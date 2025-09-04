use crate::domain::service::hotel_booking::HotelBookingService;
use async_trait::async_trait;
use shared::domain::model::order::{Order, OrderStatus, OrderType};
use shared::internal::order::command::RefundTransactionCommand;
use shared::messaging::order_status::{
    OrderStatusConsumerError, OrderStatusMessagePack, RabbitMQOrderStatusConsumer,
};
use shared::ports::order::OrderPort;
use std::sync::Arc;
use tracing::{error, info, instrument};

pub struct HotelOrderStatusConsumer<HBS, OP>
where
    HBS: HotelBookingService,
    OP: OrderPort,
{
    hotel_booking_service: Arc<HBS>,
    order_port: Arc<OP>,
}

impl<HBS, OP> HotelOrderStatusConsumer<HBS, OP>
where
    HBS: HotelBookingService,
    OP: OrderPort,
{
    pub fn new(hotel_booking_service: Arc<HBS>, order_port: Arc<OP>) -> Self {
        Self {
            hotel_booking_service,
            order_port,
        }
    }
}

#[async_trait]
impl<HBS, OP> RabbitMQOrderStatusConsumer for HotelOrderStatusConsumer<HBS, OP>
where
    HBS: HotelBookingService,
    OP: OrderPort,
{
    fn binding_key(&self) -> &'static str {
        OrderType::Hotel.message_queue_name()
    }

    #[instrument(skip(self))]
    async fn consume(
        &self,
        message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError> {
        info!("Processing Hotel order status change");

        let mut to_cancel_order_id_list = Vec::new();
        let mut to_booking_order_id_list = Vec::new();

        for message in message_pack.messages {
            if message.order_type != OrderType::Hotel {
                error!(
                    "invalid order type for hotel consumer: {}",
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
            .hotel_booking_service
            .booking_group(to_booking_order_id_list, message_pack.atomic)
            .await
            .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;

        if !tx.is_empty() {
            let tx_list_boxed = tx
                .into_iter()
                .map(|tx| Box::new(tx) as Box<dyn Order>)
                .collect::<Vec<_>>();

            self.order_port
                .refund_transaction(RefundTransactionCommand {
                    transaction_id: message_pack.transaction_uuid,
                    to_refund_orders: tx_list_boxed
                        .into_iter()
                        .map(|x| x.as_ref().into())
                        .collect(),
                })
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        for order_uuid in to_cancel_order_id_list {
            self.hotel_booking_service
                .cancel_hotel(order_uuid)
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        Ok(())
    }
}
