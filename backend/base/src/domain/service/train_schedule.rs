use crate::Verified;
use crate::domain::model::station::StationId;
use crate::domain::model::train::{TrainId, TrainNumber};
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};
use sea_orm::prelude::DateTimeWithTimeZone;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainScheduleServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid station id: {0}")]
    InvalidStationId(u64),
    #[error("invalid train id: {0}")]
    InvalidTrainId(TrainId),
    #[error("invalid train number: {0}")]
    InvalidTrainNumber(String),
}

#[async_trait]
pub trait TrainScheduleService: 'static + Send + Sync {
    async fn add_schedule(
        &self,
        train_id: TrainId,
        date: NaiveDate,
    ) -> Result<(), TrainScheduleServiceError>;

    async fn get_schedules(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError>;

    async fn get_schedule_by_train_number_and_date(
        &self,
        train_number: String,
        departure_date: NaiveDate,
    ) -> Result<Option<TrainSchedule>, TrainScheduleServiceError>;

    async fn auto_plan_schedule(
        &self,
        begin_date: NaiveDate,
        days: i32,
    ) -> Result<(), TrainScheduleServiceError>;

    async fn auto_plan_schedule_daemon(&self, days: i32);

    // async fn find_schedules(
    //     &self,
    //     date: NaiveDate,
    //     from_station: StationId,
    //     to_station: StationId,
    // ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError>;

    async fn direct_schedules(
        &self,
        date: chrono::NaiveDate,
        pairs: &[(StationId, StationId)],
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError>;

    async fn transfer_schedules(
        &self,
        date: chrono::NaiveDate,
        pairs: &[(StationId, StationId)],
    ) -> Result<Vec<(Vec<TrainScheduleId>, Option<StationId>)>, TrainScheduleServiceError>;

    async fn get_station_arrival_time(
        &self,
        train_schedule_id: TrainScheduleId,
        station_id: StationId,
    ) -> Result<sea_orm::prelude::DateTimeWithTimeZone, TrainScheduleServiceError>;

    async fn get_terminal_arrival_time(
        &self,
        train_number: TrainNumber<Verified>,
        origin_departure_time: NaiveDateTime,
    ) -> Result<DateTimeWithTimeZone, TrainScheduleServiceError>;
}
