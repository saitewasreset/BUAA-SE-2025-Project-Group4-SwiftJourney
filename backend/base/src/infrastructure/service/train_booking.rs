use crate::domain::model::order::{Order, OrderStatus, TrainOrder};
use crate::domain::repository::order::OrderRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::train_booking::{TrainBookingService, TrainBookingServiceError};
use crate::domain::service::train_seat::{TrainSeatService, TrainSeatServiceError};
use crate::domain::service::transaction::TransactionService;
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

pub struct TrainBookingServiceImpl<TSR, TSS, TS, TR, OR>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
    TS: TransactionService,
    TR: TransactionRepository,
    OR: OrderRepository,
{
    train_schedule_repository: Arc<TSR>,
    train_seat_service: Arc<TSS>,
    transaction_server: Arc<TS>,
    transaction_repository: Arc<TR>,
    order_repository: Arc<OR>,
}

impl<TSR, TSS, TS, TR, OR> TrainBookingServiceImpl<TSR, TSS, TS, TR, OR>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
    TS: TransactionService,
    TR: TransactionRepository,
    OR: OrderRepository,
{
    pub fn new(
        train_schedule_repository: Arc<TSR>,
        train_seat_service: Arc<TSS>,
        transaction_server: Arc<TS>,
        transaction_repository: Arc<TR>,
        order_repository: Arc<OR>,
    ) -> Self {
        Self {
            train_schedule_repository,
            train_seat_service,
            transaction_server,
            transaction_repository,
            order_repository,
        }
    }
}

#[async_trait]
impl<TSR, TSS, TS, TR, OR> TrainBookingService for TrainBookingServiceImpl<TSR, TSS, TS, TR, OR>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
    TS: TransactionService,
    TR: TransactionRepository,
    OR: OrderRepository,
{
    #[instrument(skip(self))]
    async fn booking_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError> {
        info!("Booking train order: {}", order_uuid);

        let mut train_order = match self
            .order_repository
            .find_train_order_by_uuid(order_uuid)
            .await
        {
            Ok(Some(order)) => order,
            Ok(None) => return Err(TrainBookingServiceError::InvalidOrder(order_uuid)),
            Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
        };

        if train_order.order_status() != OrderStatus::Paid {
            return Err(TrainBookingServiceError::InvalidOrderStatus(
                order_uuid,
                train_order.order_status(),
            ));
        }

        let train_schedule_id = train_order.train_schedule_id();
        let seat = match train_order.seat() {
            Some(seat) => seat,
            None => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Seat information is missing")),
                ));
            }
        };
        let seat_type = seat.seat_type();
        let station_range = train_order.station_range();

        let train_schedule = match self.train_schedule_repository.find(train_schedule_id).await {
            Ok(Some(schedule)) => schedule,
            Ok(None) => {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Train schedule not found")),
                ));
            }
            Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
        };

        let seat_availability_id =
            train_schedule.get_seat_availability_id(station_range, seat_type.clone());

        let seat_location_info = seat.location_info();

        let seat = match self
            .train_seat_service
            .reserve_seat(seat_availability_id, seat_location_info)
            .await
        {
            Ok(seat) => seat,
            Err(err) => {
                if let TrainSeatServiceError::NoAvailableSeat = err {
                    train_order.set_status(OrderStatus::Failed);

                    return Err(TrainBookingServiceError::NoAvailableTickets(order_uuid));
                }

                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Seat reservation failed: {}", err)),
                ));
            }
        };

        train_order.set_status(OrderStatus::Ongoing);
        train_order.set_seat(Some(seat.clone()));

        Ok(())
    }

    async fn cancel_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError> {
        let mut train_order = match self
            .order_repository
            .find_train_order_by_uuid(order_uuid)
            .await
        {
            Ok(Some(order)) => order,
            Ok(None) => return Err(TrainBookingServiceError::InvalidOrder(order_uuid)),
            Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
        };

        let status = train_order.order_status();
        // 只有Unpaid（未支付）、Paid（已支付）、Ongoing（未出行）状态的订单可以取消
        if !(status == OrderStatus::Unpaid
            || status == OrderStatus::Paid
            || status == OrderStatus::Ongoing)
        {
            return Err(TrainBookingServiceError::InvalidOrderStatus(
                order_uuid, status,
            ));
        }

        // 释放座位
        if status == OrderStatus::Ongoing {
            let train_schedule_id = train_order.train_schedule_id();

            let seat = match train_order.seat() {
                Some(seat) => seat,
                None => {
                    return Err(TrainBookingServiceError::InfrastructureError(
                        ServiceError::RelatedServiceError(anyhow!("Seat information is missing")),
                    ));
                }
            };
            let seat_type = seat.seat_type();

            let station_range = train_order.station_range();

            let train_schedule = match self.train_schedule_repository.find(train_schedule_id).await
            {
                Ok(Some(schedule)) => schedule,
                Ok(None) => {
                    return Err(TrainBookingServiceError::InfrastructureError(
                        ServiceError::RelatedServiceError(anyhow!("Train schedule not found")),
                    ));
                }
                Err(err) => return Err(TrainBookingServiceError::InfrastructureError(err.into())),
            };

            let seat_availability_id =
                train_schedule.get_seat_availability_id(station_range, seat_type.clone());

            if let Err(err) = self
                .train_seat_service
                .free_seat(seat_availability_id, seat.clone())
                .await
            {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Failed to release seat: {}", err)),
                ));
            }
        }

        train_order.set_status(OrderStatus::Cancelled);

        if status == OrderStatus::Paid || status == OrderStatus::Ongoing {
            if let Some(transaction_id) = train_order.payment_info().refund_transaction_id() {
                let transaction = match self.transaction_repository.find(transaction_id).await {
                    Ok(Some(transaction)) => transaction,
                    Ok(None) => {
                        return Err(TrainBookingServiceError::InfrastructureError(
                            ServiceError::RelatedServiceError(anyhow!("Transaction not found")),
                        ));
                    }
                    Err(err) => {
                        return Err(TrainBookingServiceError::InfrastructureError(err.into()));
                    }
                };

                let orders: Vec<Box<dyn Order>> = vec![Box::new(train_order.clone())];

                if let Err(err) = self
                    .transaction_server
                    .refund_transaction(transaction.uuid(), &orders)
                    .await
                {
                    return Err(TrainBookingServiceError::InfrastructureError(
                        ServiceError::RelatedServiceError(anyhow!(
                            "Failed to refund transaction: {}",
                            err
                        )),
                    ));
                }
            } else {
                return Err(TrainBookingServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("Paid order without transaction ID")),
                ));
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<TrainOrder>, TrainBookingServiceError> {
        info!("Booking group of train orders: {:?}", order_uuid_list);

        let mut successful_orders: Vec<TrainOrder> = Vec::new();

        for order_uuid in order_uuid_list.iter() {
            let result = self.booking_ticket(*order_uuid).await;

            if let Err(err) = result {
                error!("Failed to booking ticket {}: {}", order_uuid, err);
                if atomic {
                    for order in &successful_orders {
                        let _ = self.cancel_ticket(order.uuid()).await;
                    }
                    return Err(err);
                } else {
                    continue;
                }
            }

            match self
                .order_repository
                .find_train_order_by_uuid(*order_uuid)
                .await
            {
                Ok(Some(order)) => successful_orders.push(order),
                Ok(None) => {
                    if atomic {
                        for order in &successful_orders {
                            let _ = self.cancel_ticket(order.uuid()).await;
                        }

                        return Err(TrainBookingServiceError::InfrastructureError(
                            ServiceError::RelatedServiceError(anyhow!(
                                "Order booked but not found: {}",
                                order_uuid
                            )),
                        ));
                    }
                }
                Err(err) => {
                    error!("Failed to get train order {}: {}", order_uuid, err);

                    if atomic {
                        for order in &successful_orders {
                            let _ = self.cancel_ticket(order.uuid()).await;
                        }

                        return Err(TrainBookingServiceError::InfrastructureError(err.into()));
                    }
                }
            }
        }

        Ok(successful_orders)
    }
}
