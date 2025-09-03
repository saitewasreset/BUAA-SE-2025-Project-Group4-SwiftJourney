use crate::application::service::internal::{TrainInternalService, TrainInternalServiceError};
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use async_trait::async_trait;
use chrono::DateTime;
use shared::domain::RepositoryError;
use shared::domain::model::train::{TrainId, TrainNumber};
use shared::internal::train::command::{GetTrainByNumberQuery, GetTrainScheduleQuery};
use shared::internal::train::dto::{TerminalArrivalTimeDTO, TrainDTO, TrainScheduleDTO};
use shared::{Verified, domain::RepositoryError};
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

    #[instrument(skip(self))]
    async fn get_terminal_arrival_time(
        &self,
        query: GetTerminalArrivalTimeQuery,
    ) -> Result<Option<TerminalArrivalTimeDTO>, TrainInternalServiceError> {
        let train = self
            .train_repository
            .find_by_train_number(train_number.clone())
            .await
            .inspect_err(|e| error!("Failed to get train for verified train number: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?;

        let train_id = train.get_id().expect("train should have id");
        let origin_departure_date = origin_departure_time.date_naive();

        let train_schedule = self
            .train_schedule_repository
            .find_by_id_and_date(train_id, origin_departure_date)
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to get train schedule for train {} on date {}: {}",
                    train_id, origin_departure_date, e
                )
            })
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?
            .ok_or_else(|| {
                warn!(
                    "no train schedule found for train {} on date {}",
                    train_id, origin_departure_date
                );

                TrainScheduleServiceError::InvalidTrainNumber(train_number.to_string())
            })?;

        let route_id = train_schedule.route_id();

        let route = self
            .route_repository
            .find(route_id)
            .await
            .inspect_err(|e| error!("Failed to get route for train schedule: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?
            .ok_or(TrainScheduleServiceError::InfrastructureError(
                ServiceError::RepositoryError(RepositoryError::InconsistentState(anyhow!(
                    "no route found for route id: {}",
                    route_id
                ))),
            ))?;

        let terminal_stop = route
            .stops()
            .iter()
            .max_by(|a, b| a.order().cmp(&b.order()))
            .expect("route should have at least one stop");

        let terminal_arrival_offset = terminal_stop.arrival_time() as i64;

        let arrival_time = origin_departure_time + TimeDelta::seconds(terminal_arrival_offset);

        Ok(arrival_time)
    }
}

