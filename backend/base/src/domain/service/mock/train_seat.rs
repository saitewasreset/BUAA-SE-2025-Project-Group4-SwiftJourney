#![cfg(test)]

use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::train::SeatType;
use crate::domain::model::train_schedule::{
    Seat, SeatAvailabilityId, SeatLocationInfo, StationRange, TrainSchedule,
};
use crate::domain::service::train_seat::{TrainSeatService, TrainSeatServiceError};
use crate::Verified;
use async_trait::async_trait;
use mockall::mock;

mock! {
   pub TrainSeatService {}

    #[async_trait]
    impl TrainSeatService for TrainSeatService {
        async fn available_seats_count(
        &self,
        seat_availability_id: SeatAvailabilityId,
    ) -> Result<u32, TrainSeatServiceError>;

    async fn reserve_seat(
        &self,
        train_schedule: &mut TrainSchedule,
        station_range: StationRange<Verified>,
        seat_type: SeatType,
        seat_location_info: SeatLocationInfo,
        personal_info_id: PersonalInfoId,
    ) -> Result<Seat, TrainSeatServiceError>;

    async fn free_seat(
        &self,
        seat_availability_id: SeatAvailabilityId,
        seat: Seat,
    ) -> Result<(), TrainSeatServiceError>;
    }
}

pub fn mock_train_seat_service() -> MockTrainSeatService {
    MockTrainSeatService::new()
}
