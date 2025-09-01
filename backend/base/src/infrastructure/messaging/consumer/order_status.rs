use crate::domain::model::order::{Order, OrderStatus, OrderType};
use crate::domain::service::dish_booking::DishBookingService;
use crate::domain::service::hotel_booking::HotelBookingService;
use crate::domain::service::order_status::{
    OrderStatusConsumer, OrderStatusConsumerError, OrderStatusMessagePack,
};
use crate::domain::service::takeaway_booking::TakeawayBookingService;
use crate::domain::service::train_booking::TrainBookingService;
use crate::domain::service::transaction::TransactionService;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, instrument};

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

pub struct HotelOrderStatusConsumer<HBS, TS>
where
    HBS: HotelBookingService,
    TS: TransactionService,
{
    hotel_booking_service: Arc<HBS>,
    transaction_service: Arc<TS>,
}

impl<HBS, TS> HotelOrderStatusConsumer<HBS, TS>
where
    HBS: HotelBookingService,
    TS: TransactionService,
{
    pub fn new(hotel_booking_service: Arc<HBS>, transaction_service: Arc<TS>) -> Self {
        Self {
            hotel_booking_service,
            transaction_service,
        }
    }
}

pub struct DishOrderStatusConsumer<DBS, TS>
where
    DBS: DishBookingService,
    TS: TransactionService,
{
    dish_booking_service: Arc<DBS>,
    transaction_service: Arc<TS>,
}

impl<DBS, TS> DishOrderStatusConsumer<DBS, TS>
where
    DBS: DishBookingService,
    TS: TransactionService,
{
    pub fn new(dish_booking_service: Arc<DBS>, transaction_service: Arc<TS>) -> Self {
        Self {
            dish_booking_service,
            transaction_service,
        }
    }
}

pub struct TakeawayOrderStatusConsumer<TBS, TS>
where
    TBS: TakeawayBookingService,
    TS: TransactionService,
{
    takeaway_booking_service: Arc<TBS>,
    transaction_service: Arc<TS>,
}

impl<TBS, TS> TakeawayOrderStatusConsumer<TBS, TS>
where
    TBS: TakeawayBookingService,
    TS: TransactionService,
{
    pub fn new(takeaway_booking_service: Arc<TBS>, transaction_service: Arc<TS>) -> Self {
        Self {
            takeaway_booking_service,
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
                let tx_list_boxed = tx
                    .into_iter()
                    .map(|tx| Box::new(tx) as Box<dyn Order>)
                    .collect::<Vec<_>>();

                self.transaction_service
                    .refund_transaction(message_pack.transaction_uuid, &tx_list_boxed)
                    .await
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

#[async_trait]
impl<HBS, TS> RabbitMQOrderStatusConsumer for HotelOrderStatusConsumer<HBS, TS>
where
    HBS: HotelBookingService,
    TS: TransactionService,
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

            self.transaction_service
                .refund_transaction(message_pack.transaction_uuid, &tx_list_boxed)
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

#[async_trait]
impl<DBS, TS> RabbitMQOrderStatusConsumer for DishOrderStatusConsumer<DBS, TS>
where
    DBS: DishBookingService,
    TS: TransactionService,
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
            self.dish_booking_service
                .cancel_dish(order_uuid)
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        Ok(())
    }
}

#[async_trait]
impl<TBS, TS> RabbitMQOrderStatusConsumer for TakeawayOrderStatusConsumer<TBS, TS>
where
    TBS: TakeawayBookingService,
    TS: TransactionService,
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
            self.takeaway_booking_service
                .cancel_takeaway(order_uuid)
                .await
                .map_err(|e| OrderStatusConsumerError::RelatedServiceError(e.into()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::domain::model::order::{
        BaseOrder, DishOrder, OrderStatus, OrderTimeInfo, OrderType, PaymentInfo, TrainOrder,
    };
    use crate::domain::model::personal_info::PreferredSeatLocation;
    use crate::domain::model::train::{SeatType, SeatTypeName};
    use crate::domain::model::train_schedule::{Seat, SeatLocationInfo, SeatStatus, StationRange};
    use crate::domain::model::transaction::Transaction;
    use crate::domain::service::hotel_booking::HotelBookingServiceError;
    use crate::domain::service::order_status::{
        OrderStatusConsumerError, OrderStatusMessage, OrderStatusMessagePack,
    };

    use crate::domain::service::mock::{
        dish_booking::MockDishBookingService, hotel_booking::MockHotelBookingService,
        takeaway_booking::MockTakeawayBookingService, train_booking::MockTrainBookingService,
        transaction::MockTransactionService,
    };
    use crate::domain::service::takeaway_booking::TakeawayBookingServiceError;
    use crate::domain::service::ServiceError;

    // -------------------- TrainOrderStatusConsumer --------------------
    #[tokio::test]
    async fn test_train_consumer_success_booking_and_cancel() {
        let mut mock_train = MockTrainBookingService::new();
        let mut mock_tx = MockTransactionService::new();

        let order_id1 = Uuid::new_v4();
        let order_id2 = Uuid::new_v4();

        mock_train
            .expect_booking_group()
            .returning(|_, _| Ok(vec![]));
        mock_train.expect_cancel_ticket().returning(|_| Ok(()));
        mock_tx
            .expect_refund_transaction()
            .returning(move |_, _| Ok(order_id1));

        let consumer = TrainOrderStatusConsumer::new(Arc::new(mock_train), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid: Uuid::new_v4(),
            atomic: true,
            messages: vec![
                OrderStatusMessage {
                    order_id: order_id1,
                    order_type: OrderType::Train,
                    new_status: OrderStatus::Paid,
                },
                OrderStatusMessage {
                    order_id: order_id2,
                    order_type: OrderType::Train,
                    new_status: OrderStatus::Cancelled,
                },
            ],
        };

        assert!(consumer.consume(msg_pack).await.is_ok());
    }

    #[tokio::test]
    async fn test_train_consumer_invalid_order_type() {
        let mut mock_train = MockTrainBookingService::new();
        let mut mock_tx = MockTransactionService::new();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let train_order = TrainOrder::new(
            base_order,
            1u64.into(),
            Some(Seat::new(
                1u64.into(),
                SeatType::new(
                    Some(1u64.into()),
                    SeatTypeName::from_unchecked("二等座".to_string()),
                    1,
                    Decimal::new(1000, 2),
                ),
                SeatLocationInfo {
                    carriage: 3,
                    row: 11,
                    location: 'A',
                },
                SeatStatus::Occupied,
            )),
            SeatTypeName::from_unchecked("二等座".to_string()),
            Some(PreferredSeatLocation::A),
            StationRange::from_unchecked(1u64.into(), 1u64.into()),
        );

        mock_train
            .expect_booking_group()
            .returning(move |_, _| Ok(vec![train_order.clone()]));

        let transaction_uuid = Uuid::new_v4();

        mock_tx
            .expect_refund_transaction()
            .returning(move |_, _| Ok(transaction_uuid));

        let consumer = TrainOrderStatusConsumer::new(Arc::new(mock_train), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid,
            atomic: false,
            messages: vec![OrderStatusMessage {
                order_id: Uuid::new_v4(),
                order_type: OrderType::Hotel,
                new_status: OrderStatus::Paid,
            }],
        };

        assert!(consumer.consume(msg_pack).await.is_ok());
    }

    // -------------------- HotelOrderStatusConsumer --------------------
    #[tokio::test]
    async fn test_hotel_consumer_success() {
        let mut mock_hotel = MockHotelBookingService::new();
        let mut mock_tx = MockTransactionService::new();

        let order_id1 = Uuid::new_v4();
        let order_id2 = Uuid::new_v4();

        mock_hotel
            .expect_booking_group()
            .returning(|_, _| Ok(vec![]));
        mock_hotel.expect_cancel_hotel().returning(|_| Ok(()));
        mock_tx
            .expect_refund_transaction()
            .returning(move |_, _| Ok(order_id1));

        let consumer = HotelOrderStatusConsumer::new(Arc::new(mock_hotel), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid: Uuid::new_v4(),
            atomic: true,
            messages: vec![
                OrderStatusMessage {
                    order_id: order_id1,
                    order_type: OrderType::Hotel,
                    new_status: OrderStatus::Paid,
                },
                OrderStatusMessage {
                    order_id: order_id2,
                    order_type: OrderType::Hotel,
                    new_status: OrderStatus::Cancelled,
                },
            ],
        };

        assert!(consumer.consume(msg_pack).await.is_ok());
    }

    #[tokio::test]
    async fn test_hotel_consumer_related_service_error() {
        let mut mock_hotel = MockHotelBookingService::new();
        let mock_tx = MockTransactionService::new();

        mock_hotel.expect_booking_group().returning(|_, _| {
            Err(HotelBookingServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!("some error")),
            ))
        });

        let consumer = HotelOrderStatusConsumer::new(Arc::new(mock_hotel), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid: Uuid::new_v4(),
            atomic: false,
            messages: vec![OrderStatusMessage {
                order_id: Uuid::new_v4(),
                order_type: OrderType::Hotel,
                new_status: OrderStatus::Paid,
            }],
        };

        assert!(matches!(
            consumer.consume(msg_pack).await,
            Err(OrderStatusConsumerError::RelatedServiceError(_))
        ));
    }

    // -------------------- DishOrderStatusConsumer --------------------
    #[tokio::test]
    async fn test_dish_consumer_success() {
        let mut mock_dish = MockDishBookingService::new();
        let mut mock_tx = MockTransactionService::new();

        let order_id1 = Uuid::new_v4();
        let order_id2 = Uuid::new_v4();

        mock_dish
            .expect_booking_group()
            .returning(|_, _| Ok(vec![]));
        mock_dish.expect_cancel_dish().returning(|_| Ok(()));
        mock_tx
            .expect_refund_transaction()
            .returning(move |_, _| Ok(order_id1));

        let consumer = DishOrderStatusConsumer::new(Arc::new(mock_dish), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid: Uuid::new_v4(),
            atomic: true,
            messages: vec![
                OrderStatusMessage {
                    order_id: order_id1,
                    order_type: OrderType::Dish,
                    new_status: OrderStatus::Unpaid,
                },
                OrderStatusMessage {
                    order_id: order_id2,
                    order_type: OrderType::Dish,
                    new_status: OrderStatus::Cancelled,
                },
            ],
        };

        assert!(consumer.consume(msg_pack).await.is_ok());
    }

    #[tokio::test]
    async fn test_dish_consumer_unexpected_status() {
        let mut mock_dish = MockDishBookingService::new();
        let mut mock_tx = MockTransactionService::new();

        let train_order_id = 1u64.into();
        let dish_id = 1u64.into();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let dish_order = DishOrder::new(
            base_order.clone(),
            train_order_id,
            dish_id,
            Decimal::new(1000, 2),
            Decimal::ONE,
        );

        let transaction_id = Uuid::new_v4();

        mock_dish
            .expect_booking_group()
            .returning(move |_, _| Ok(vec![dish_order.clone()]));

        mock_tx
            .expect_refund_transaction()
            .returning(move |_, _| Ok(transaction_id));

        let consumer = DishOrderStatusConsumer::new(Arc::new(mock_dish), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid: transaction_id,
            atomic: false,
            messages: vec![OrderStatusMessage {
                order_id: Uuid::new_v4(),
                order_type: OrderType::Dish,
                new_status: OrderStatus::Unpaid,
            }],
        };

        assert!(consumer.consume(msg_pack).await.is_ok());
    }

    // -------------------- TakeawayOrderStatusConsumer --------------------
    #[tokio::test]
    async fn test_takeaway_consumer_success() {
        let mut mock_takeaway = MockTakeawayBookingService::new();
        let mut mock_tx = MockTransactionService::new();

        let order_id1 = Uuid::new_v4();
        let order_id2 = Uuid::new_v4();

        mock_takeaway
            .expect_booking_group()
            .returning(|_, _| Ok(vec![]));
        mock_takeaway.expect_cancel_takeaway().returning(|_| Ok(()));
        mock_tx
            .expect_refund_transaction()
            .returning(move |_, _| Ok(order_id1));

        let consumer = TakeawayOrderStatusConsumer::new(Arc::new(mock_takeaway), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid: Uuid::new_v4(),
            atomic: true,
            messages: vec![
                OrderStatusMessage {
                    order_id: order_id1,
                    order_type: OrderType::Takeaway,
                    new_status: OrderStatus::Unpaid,
                },
                OrderStatusMessage {
                    order_id: order_id2,
                    order_type: OrderType::Takeaway,
                    new_status: OrderStatus::Cancelled,
                },
            ],
        };

        assert!(consumer.consume(msg_pack).await.is_ok());
    }

    #[tokio::test]
    async fn test_takeaway_consumer_booking_group_error() {
        let mut mock_takeaway = MockTakeawayBookingService::new();
        let mock_tx = MockTransactionService::new();

        mock_takeaway.expect_booking_group().returning(|_, _| {
            Err(TakeawayBookingServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!("some error")),
            ))
        });

        let consumer = TakeawayOrderStatusConsumer::new(Arc::new(mock_takeaway), Arc::new(mock_tx));

        let msg_pack = OrderStatusMessagePack {
            transaction_uuid: Uuid::new_v4(),
            atomic: false,
            messages: vec![OrderStatusMessage {
                order_id: Uuid::new_v4(),
                order_type: OrderType::Takeaway,
                new_status: OrderStatus::Unpaid,
            }],
        };

        assert!(matches!(
            consumer.consume(msg_pack).await,
            Err(OrderStatusConsumerError::RelatedServiceError(_))
        ));
    }
}
