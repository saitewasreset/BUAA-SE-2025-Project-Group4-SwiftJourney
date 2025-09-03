use async_trait::async_trait;
use shared::internal::train::command::{
    GetTerminalArrivalTimeQuery, GetTrainByNumberQuery, GetTrainScheduleQuery,
};
use shared::internal::train::dto::{TerminalArrivalTimeDTO, TrainDTO, TrainScheduleDTO};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainInternalServiceError {
    #[error("invalid date time format: {0}")]
    InvalidDateTimeFormat(String),
    #[error(transparent)]
    RelatedServiceError(#[from] anyhow::Error),
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
    ) -> Result<Option<TerminalArrivalTimeDTO>, TrainInternalServiceError>;
}
