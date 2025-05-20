use crate::domain::model::order::TrainOrder;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::train_booking::{TrainBookingService, TrainBookingServiceError};
use crate::domain::service::train_seat::TrainSeatService;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

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
    async fn booking_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError> {
        todo!()
    }

    async fn cancel_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError> {
        todo!()
    }

    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<TrainOrder>, TrainBookingServiceError> {
        todo!()
    }
}
