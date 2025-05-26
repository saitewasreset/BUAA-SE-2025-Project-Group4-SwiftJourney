use crate::domain::model::station::StationId;
use crate::domain::model::train::TrainId;
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use chrono::NaiveDate;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainScheduleServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid station id: {0}")]
    InvalidStationId(u64),
}

#[async_trait]
pub trait TrainScheduleService {
    async fn add_schedule(
        &self,
        train_id: TrainId,
        date: NaiveDate,
    ) -> Result<(), TrainScheduleServiceError>;

    async fn get_schedules(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError>;

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
}
