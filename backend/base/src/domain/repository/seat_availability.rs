use crate::Verified;
use crate::domain::model::train::SeatTypeId;
use crate::domain::model::train_schedule::{SeatAvailability, StationRange, TrainScheduleId};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use std::collections::HashMap;

// seat_type_id -> (begin_station_id, end_station_id) -> [seat_id]
pub type OccupiedSeatInfoMap = HashMap<i32, HashMap<(i32, i32), Vec<i64>>>;

#[async_trait]
pub trait SeatAvailabilityRepository: Repository<SeatAvailability> {
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
