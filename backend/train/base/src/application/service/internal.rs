use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use shared::application_error::ApplicationError;
use shared::internal::train::command::{
    GetTerminalArrivalTimeQuery, GetTrainByNumberQuery, GetTrainScheduleQuery,
    VerifyTrainNumberQuery,
};
use shared::internal::train::dto::{
    DbRouteDTO, DbSeatTypeDTO, DbSeatTypeMappingDTO, DbTrainDTO, DbTrainScheduleDTO, TrainDTO,
    TrainScheduleDTO,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainInternalServiceError {
    #[error("invalid date time format: {0}")]
    InvalidDateTimeFormat(String),

    #[error("invalid train number: {0}")]
    InvalidTrainNumber(String),

    #[error(transparent)]
    RelatedServiceError(#[from] anyhow::Error),
}

impl ApplicationError for TrainInternalServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainInternalServiceError::InvalidDateTimeFormat(_) => 92001,
            TrainInternalServiceError::InvalidTrainNumber(_) => 92002,
            TrainInternalServiceError::RelatedServiceError(_) => 92003,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait TrainInternalService: 'static + Send + Sync {
    async fn get_train_by_number(
        &self,
        query: GetTrainByNumberQuery,
    ) -> Result<Option<TrainDTO>, TrainInternalServiceError>;

    async fn get_train_schedule(
        &self,
        query: GetTrainScheduleQuery,
    ) -> Result<Option<TrainScheduleDTO>, TrainInternalServiceError>;

    async fn get_terminal_arrival_time(
        &self,
        query: GetTerminalArrivalTimeQuery,
    ) -> Result<DateTime<FixedOffset>, TrainInternalServiceError>;

    async fn get_trains(&self) -> Result<Vec<TrainDTO>, TrainInternalServiceError>;

    async fn verify_train_number(
        &self,
        query: VerifyTrainNumberQuery,
    ) -> Result<bool, TrainInternalServiceError>;

    async fn db_get_trains(&self) -> Result<Vec<DbTrainDTO>, TrainInternalServiceError>;

    async fn db_get_routes(&self) -> Result<Vec<DbRouteDTO>, TrainInternalServiceError>;

    async fn db_get_train_schedule(
        &self,
    ) -> Result<Vec<DbTrainScheduleDTO>, TrainInternalServiceError>;

    async fn db_get_seat_type(&self) -> Result<Vec<DbSeatTypeDTO>, TrainInternalServiceError>;

    async fn db_get_seat_type_mapping(
        &self,
    ) -> Result<Vec<DbSeatTypeMappingDTO>, TrainInternalServiceError>;
}
