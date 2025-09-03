#![cfg(test)]

use crate::domain::model::train::SeatTypeId;
use crate::domain::model::train_schedule::{
    SeatAvailability, SeatAvailabilityId, StationRange, TrainScheduleId,
};
use crate::domain::repository::seat_availability::{
    OccupiedSeatInfoMap, SeatAvailabilityRepository,
};
use crate::domain::{Repository, RepositoryError};
use crate::Verified;
use async_trait::async_trait;
use mockall::mock;

mock! {
    pub SeatAvailabilityRepository {}

    #[async_trait]
    impl Repository<SeatAvailability> for SeatAvailabilityRepository {
        async fn find(&self, id: SeatAvailabilityId) -> Result<Option<SeatAvailability>, RepositoryError>;
        async fn remove(&self, aggregate: SeatAvailability) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut SeatAvailability) -> Result<SeatAvailabilityId, RepositoryError>;
    }

    #[async_trait]
    impl SeatAvailabilityRepository for SeatAvailabilityRepository {
        async fn get_train_schedule_seat_availability_list(
        &self,
        train_schedule_id: TrainScheduleId,
    ) -> Result<Vec<crate::models::seat_availability::Model>, RepositoryError>;

    async fn get_train_schedule_occupied_seat(
        &self,
        train_schedule_id: TrainScheduleId,
    ) -> Result<OccupiedSeatInfoMap, RepositoryError>;

    async fn find_by_schedule_seat_type_station_range(
        &self,
        train_schedule_id: TrainScheduleId,
        seat_type_id: SeatTypeId,
        station_range: StationRange<Verified>,
    ) -> Result<Option<SeatAvailability>, RepositoryError>;
    }
}

pub fn mock_seat_availability_repository() -> MockSeatAvailabilityRepository {
    MockSeatAvailabilityRepository::new()
}
