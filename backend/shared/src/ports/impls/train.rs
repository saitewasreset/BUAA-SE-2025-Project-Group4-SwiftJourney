use crate::api::{ApiEndpoint, InternalApiError, SuperClient, TrainInternalServiceApi};
use crate::internal::train::command::{
    GetTerminalArrivalTimeQuery, GetTrainByNumberQuery, GetTrainScheduleQuery,
    VerifyTrainNumberQuery,
};
use crate::internal::train::dto::{TrainDTO, TrainScheduleDTO};
use crate::ports::train::TrainPort;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use tracing::error;

pub struct HttpTrainPortImpl {
    super_client: SuperClient,
}

impl HttpTrainPortImpl {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);

        Self { super_client }
    }
}

#[async_trait]
impl TrainPort for HttpTrainPortImpl {
    async fn get_train_by_number(
        &self,
        query: GetTrainByNumberQuery,
    ) -> Result<Option<TrainDTO>, InternalApiError> {
        self.super_client
            .post(TrainInternalServiceApi::GetTrainByNumber, query)
            .await
            .inspect_err(|e| error!("Failed to get train by number: {:?}", e))
    }

    async fn get_train_schedule(
        &self,
        query: GetTrainScheduleQuery,
    ) -> Result<Option<TrainScheduleDTO>, InternalApiError> {
        self.super_client
            .post(
                TrainInternalServiceApi::GetTrainScheduleByTrainIdAndOriginDepartureTime,
                query,
            )
            .await
            .inspect_err(|e| error!("Failed to get train schedule: {:?}", e))
    }

    async fn get_terminal_arrival_time(
        &self,
        query: GetTerminalArrivalTimeQuery,
    ) -> Result<DateTime<FixedOffset>, InternalApiError> {
        self.super_client
            .post(TrainInternalServiceApi::GetTerminalArrivalTime, query)
            .await
            .inspect_err(|e| error!("Failed to get terminal arrival time: {:?}", e))
    }

    async fn get_trains(&self) -> Result<Vec<TrainDTO>, InternalApiError> {
        self.super_client
            .get(TrainInternalServiceApi::GetTrains)
            .await
            .inspect_err(|e| error!("Failed to get trains: {:?}", e))
    }

    async fn verify_train_number(
        &self,
        query: VerifyTrainNumberQuery,
    ) -> Result<bool, InternalApiError> {
        self.super_client
            .post(TrainInternalServiceApi::VerifyTrainNumber, query)
            .await
            .inspect_err(|e| error!("Failed to verify train number: {:?}", e))
    }
}
