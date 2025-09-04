use crate::api::InternalApiError;
use crate::internal::train::command::{
    GetTerminalArrivalTimeQuery, GetTrainByNumberQuery, GetTrainScheduleQuery,
    VerifyTrainNumberQuery,
};
use crate::internal::train::dto::{
    DbRouteDTO, DbSeatTypeDTO, DbSeatTypeMappingDTO, DbTrainDTO, DbTrainScheduleDTO, TrainDTO,
    TrainScheduleDTO,
};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};

#[async_trait]
pub trait TrainPort: 'static + Send + Sync {
    async fn get_train_by_number(
        &self,
        query: GetTrainByNumberQuery,
    ) -> Result<Option<TrainDTO>, InternalApiError>;

    async fn get_train_schedule(
        &self,
        query: GetTrainScheduleQuery,
    ) -> Result<Option<TrainScheduleDTO>, InternalApiError>;

    async fn get_terminal_arrival_time(
        &self,
        query: GetTerminalArrivalTimeQuery,
    ) -> Result<DateTime<FixedOffset>, InternalApiError>;

    async fn get_trains(&self) -> Result<Vec<TrainDTO>, InternalApiError>;

    async fn verify_train_number(
        &self,
        query: VerifyTrainNumberQuery,
    ) -> Result<bool, InternalApiError>;

    async fn db_get_trains(&self) -> Result<Vec<DbTrainDTO>, InternalApiError>;

    async fn db_get_routes(&self) -> Result<Vec<DbRouteDTO>, InternalApiError>;

    async fn db_get_train_schedule(&self) -> Result<Vec<DbTrainScheduleDTO>, InternalApiError>;

    async fn db_get_seat_type(&self) -> Result<Vec<DbSeatTypeDTO>, InternalApiError>;

    async fn db_get_seat_type_mapping(&self)
    -> Result<Vec<DbSeatTypeMappingDTO>, InternalApiError>;
}
