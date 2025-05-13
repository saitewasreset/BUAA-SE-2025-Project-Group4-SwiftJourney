use crate::domain::model::order::{Order, OrderStatus, OrderType};
use crate::domain::service::order_status::{
    OrderStatusConsumer, OrderStatusConsumerError, OrderStatusMessagePack,
};
use crate::domain::service::train_booking::TrainBookingService;
use crate::domain::service::transaction::TransactionService;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, instrument};

#[async_trait]
pub trait RabbitMQOrderStatusConsumer: 'static + Send + Sync {
    fn binding_key(&self) -> &'static str;
    async fn consume(
        &self,
        message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError>;
}

#[async_trait]
impl<T> OrderStatusConsumer for T
where
    T: RabbitMQOrderStatusConsumer,
{
    async fn consume_order_status_change(
        &self,
        order_status_message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError> {
        self.consume(order_status_message_pack).await
    }
}

pub struct TrainOrderStatusConsumer<TBS, TS>
where
    TBS: TrainBookingService,
    TS: TransactionService,
{
    train_booking_service: Arc<TBS>,
    transaction_service: Arc<TS>,
}

impl<TBS, TS> TrainOrderStatusConsumer<TBS, TS>
where
    TBS: TrainBookingService,
    TS: TransactionService,
{
    pub fn new(train_booking_service: Arc<TBS>, transaction_service: Arc<TS>) -> Self {
        Self {
            train_booking_service,
            transaction_service,
        }
    }
}

#[async_trait]
impl<TBS, TS> RabbitMQOrderStatusConsumer for TrainOrderStatusConsumer<TBS, TS>
where
    TBS: TrainBookingService,
    TS: TransactionService,
{
    fn binding_key(&self) -> &'static str {
        OrderType::Train.message_queue_name()
    }

    #[instrument(skip(self))]
    async fn consume(
        &self,
        message_pack: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusConsumerError> {
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

        let tx = self
            .train_booking_service
            .booking_group(to_booking_order_id_list, message_pack.atomic)
            .await
            .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;

        if !tx.is_empty() {
            let tx_list_boxed = tx
                .into_iter()
                .map(|tx| Box::new(tx) as Box<dyn Order>)
                .collect::<Vec<_>>();

            self.transaction_service
                .refund_transaction(message_pack.transaction_uuid, &tx_list_boxed)
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
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
