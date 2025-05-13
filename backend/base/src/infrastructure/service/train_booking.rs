use crate::domain::model::order::{Order, OrderStatus, TrainOrder};
use crate::domain::model::transaction::{Transaction, TransactionId};
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::train_booking::{TrainBookingService, TrainBookingServiceError};
use crate::domain::service::train_seat::TrainSeatService;
use crate::domain::{Repository, RepositoryError};
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;

pub struct TrainBookingServiceImpl<TSR, TSS>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
{
    train_schedule_repository: Arc<TSR>,
    train_seat_service: Arc<TSS>,
}

#[async_trait]
impl<TSR, TSS> TrainBookingService for TrainBookingServiceImpl<TSR, TSS>
where
    TSR: TrainScheduleRepository,
    TSS: TrainSeatService,
{
    async fn booking_ticket(&self, order: TrainOrder) -> Result<(), TrainBookingServiceError> {
        if order.order_status() != OrderStatus::Paid {
            return Err(TrainBookingServiceError::InvalidOrderStatus(
                order.uuid(),
                order.order_status(),
            ));
        }

        let train_schedule = self
            .train_schedule_repository
            .find(order.train_schedule_id())
            .await
            .map_err(|e| TrainBookingServiceError::InfrastructureError(e.into()))?
            .ok_or(TrainBookingServiceError::InfrastructureError(
                RepositoryError::InconsistentState(anyhow!(
                    "No train schedule for order uuid: {}",
                    order.uuid()
                ))
                .into(),
            ))?;

        todo!()
    }

    async fn cancel_ticket(&self, order: TrainOrder) -> Result<(), TrainBookingServiceError> {
        todo!()
    }

    async fn booking_group(
        &self,
        transaction_id: TransactionId,
    ) -> Result<Option<Transaction>, TrainBookingServiceError> {
        todo!()
    }
}
