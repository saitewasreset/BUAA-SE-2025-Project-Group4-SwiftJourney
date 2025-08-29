#![cfg(test)]

use crate::domain::model::train::TrainId;
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate};
use mockall::mock;
use crate::domain::repository::train_schedule::TrainScheduleRepository;

mock! {
    pub TrainScheduleRepository {}

    #[async_trait]
    impl Repository<TrainSchedule> for TrainScheduleRepository {
        async fn find(&self, id: TrainScheduleId) -> Result<Option<TrainSchedule>, RepositoryError>;
        async fn remove(&self, aggregate: TrainSchedule) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut TrainSchedule) -> Result<TrainScheduleId, RepositoryError>;
    }

    #[async_trait]
    impl TrainScheduleRepository for TrainScheduleRepository {
        async fn find_by_date(&self, date: NaiveDate) -> Result<Vec<TrainSchedule>, RepositoryError>;

    async fn find_by_id_and_date(
        &self,
        train_id: TrainId,
        date: NaiveDate,
    ) -> Result<Option<TrainSchedule>, RepositoryError>;

    async fn find_by_train_id(
        &self,
        train_id: TrainId,
    ) -> Result<Vec<TrainSchedule>, RepositoryError>;

    async fn find_by_train_id_and_origin_departure_time(
        &self,
        train_id: TrainId,
        origin_departure_time: DateTime<FixedOffset>,
    ) -> Result<Option<TrainSchedule>, RepositoryError>;

    async fn save_many_no_conflict(
        &self,
        schedules: Vec<TrainSchedule>,
    ) -> Result<(), RepositoryError>;

    async fn get_latest_schedule_date(&self) -> Result<Option<NaiveDate>, RepositoryError>;
    }
}

pub fn mock_train_schedule_repository() -> MockTrainScheduleRepository {
    MockTrainScheduleRepository::new()
}