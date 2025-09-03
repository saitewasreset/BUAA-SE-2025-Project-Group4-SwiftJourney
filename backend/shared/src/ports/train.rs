use crate::api::InternalApiError;
use crate::internal::train::command::{
    GetTerminalArrivalTimeQuery, GetTrainByNumberQuery, GetTrainScheduleQuery,
};
use crate::internal::train::dto::{TerminalArrivalTimeDTO, TrainDTO, TrainScheduleDTO};
use async_trait::async_trait;

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
    ) -> Result<Option<TerminalArrivalTimeDTO>, InternalApiError>;
}
