use crate::domain::model::train_schedule::{SeatAvailability, TrainScheduleId};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

#[async_trait]
pub trait SeatAvailabilityRepository: Repository<SeatAvailability> {
    async fn get_train_schedule_seat_availability_list(
        &self,
        train_schedule_id: TrainScheduleId,
    ) -> Result<Vec<crate::models::seat_availability::Model>, RepositoryError>;
}
