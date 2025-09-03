use crate::application::service::internal::{TrainInternalService, TrainInternalServiceError};
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use async_trait::async_trait;
use chrono::DateTime;
use shared::domain::RepositoryError;
use shared::domain::model::train::{TrainId, TrainNumber};
use shared::internal::train::command::{GetTrainByNumberQuery, GetTrainScheduleQuery};
use shared::internal::train::dto::{TrainDTO, TrainScheduleDTO};
use std::sync::Arc;
use tracing::{error, instrument};

pub struct TrainInternalServiceImpl<TR, TSR>
where
    TR: TrainRepository,
    TSR: TrainScheduleRepository,
{
    train_repository: Arc<TR>,
    train_schedule_repository: Arc<TSR>,
}

impl<TR, TSR> TrainInternalServiceImpl<TR, TSR>
where
    TR: TrainRepository,
    TSR: TrainScheduleRepository,
{
    pub fn new(train_repository: Arc<TR>, train_schedule_repository: Arc<TSR>) -> Self {
        Self {
            train_repository,
            train_schedule_repository,
        }
    }
}

#[async_trait]
impl<TR, TSR> TrainInternalService for TrainInternalServiceImpl<TR, TSR>
where
    TR: TrainRepository,
    TSR: TrainScheduleRepository,
{
    #[instrument(skip(self))]
    async fn get_train_by_number(
        &self,
        query: GetTrainByNumberQuery,
    ) -> Result<Option<TrainDTO>, TrainInternalServiceError> {
        let train_number = TrainNumber::from_unchecked(query.train_number);
        let result = self
            .train_repository
            .find_by_train_number(train_number)
            .await;

        match result {
            Ok(train) => Ok(Some(train.into())),
            Err(RepositoryError::InconsistentState(_)) => Ok(None),
            Err(e) => {
                error!("error getting train by number: {:?}", e);
                Err(TrainInternalServiceError::RelatedServiceError(e.into()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_train_schedule(
        &self,
        query: GetTrainScheduleQuery,
    ) -> Result<Option<TrainScheduleDTO>, TrainInternalServiceError> {
        let train_id = TrainId::from(query.train_id);
        let origin_departure_time = DateTime::parse_from_rfc3339(&query.origin_departure_time)
            .map_err(|_| {
                TrainInternalServiceError::InvalidDateTimeFormat(query.origin_departure_time)
            })?;

        let schedule = self
            .train_schedule_repository
            .find_by_train_id_and_origin_departure_time(train_id, origin_departure_time.into())
            .await
            .inspect_err(|e| error!("error getting train schedule: {:?}", e))
            .map_err(|e| TrainInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(schedule.map(|s| s.into()))
    }
}
