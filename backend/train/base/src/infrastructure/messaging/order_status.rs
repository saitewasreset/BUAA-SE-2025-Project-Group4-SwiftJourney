use crate::domain::service::train_booking::TrainBookingService;
use async_trait::async_trait;
use shared::domain::model::order::{Order, OrderStatus, OrderType};
use shared::internal::order::command::RefundTransactionCommand;
use shared::internal::order::dto::InternalOrderDTO;
use shared::messaging::order_status::{
    OrderStatusConsumerError, OrderStatusMessagePack, RabbitMQOrderStatusConsumer,
};
use shared::ports::order::OrderPort;
use std::sync::Arc;
use tracing::{error, info, instrument};

pub struct TrainOrderStatusConsumer<TBS, OP>
where
    TBS: TrainBookingService,
    OP: OrderPort,
{
    train_booking_service: Arc<TBS>,
    order_port: Arc<OP>,
}

impl<TBS, OP> TrainOrderStatusConsumer<TBS, OP>
where
    TBS: TrainBookingService,
    OP: OrderPort,
{
    pub fn new(train_booking_service: Arc<TBS>, order_port: Arc<OP>) -> Self {
        Self {
            train_booking_service,
            order_port,
        }
    }
}

#[async_trait]
impl<TBS, OP> RabbitMQOrderStatusConsumer for TrainOrderStatusConsumer<TBS, OP>
where
    TBS: TrainBookingService,
    OP: OrderPort,
{
    fn binding_key(&self) -> &'static str {
        OrderType::Train.message_queue_name()
    }

    #[instrument(skip(self))]
    async fn consume(
        &self,
        message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError> {
        info!("Processing Train order status change");

        let mut to_cancel_order_id_list = Vec::new();
        let mut to_booking_order_id_list = Vec::new();

        for message in message_pack.messages {
            if message.order_type != OrderType::Train {
                error!(
                    "invalid order type for train order consumer: {}",
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

        if !to_booking_order_id_list.is_empty() {
            let tx = self
                .train_booking_service
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
                    .inspect_err(|e| error!("Failed to send refund transaction: {:?}", e))
                    .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
            }
        }

        for order_uuid in to_cancel_order_id_list {
            self.train_booking_service
                .cancel_ticket(order_uuid)
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        Ok(())
    }
}
