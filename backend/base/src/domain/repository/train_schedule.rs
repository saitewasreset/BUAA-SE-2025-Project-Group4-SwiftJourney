use crate::domain::model::train::TrainId;
use crate::domain::model::train_schedule::TrainSchedule;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate};

#[async_trait]
pub trait TrainScheduleRepository: Repository<TrainSchedule> {
    async fn find_by_date(&self, date: NaiveDate) -> Result<Vec<TrainSchedule>, RepositoryError>;
    async fn find_by_train_id(
        &self,
        train_id: TrainId,
    ) -> Result<Vec<TrainSchedule>, RepositoryError>;

    async fn find_by_train_id_and_origin_departure_time(
        &self,
        train_id: TrainId,
        origin_departure_time: DateTime<FixedOffset>,
    ) -> Result<Option<TrainSchedule>, RepositoryError>;
}
