use crate::application::service::internal::{TrainInternalService, TrainInternalServiceError};
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::train_type::{
    TrainTypeConfigurationService, TrainTypeConfigurationServiceError,
};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, TimeDelta};
use shared::domain::RepositoryError;
use shared::domain::model::train::{TrainId, TrainNumber};
use shared::domain::{Identifiable, ServiceError};
use shared::internal::train::command::{
    GetTerminalArrivalTimeQuery, GetTrainByNumberQuery, GetTrainScheduleQuery,
    VerifyTrainNumberQuery,
};
use shared::internal::train::dto::{DbRouteDTO, DbTrainDTO, TrainDTO, TrainScheduleDTO};
use std::sync::Arc;
use tracing::{error, instrument, warn};

pub struct TrainInternalServiceImpl<TR, TTS, TSR, RR>
where
    TR: TrainRepository,
    TTS: TrainTypeConfigurationService,
    TSR: TrainScheduleRepository,
    RR: RouteRepository,
{
    train_repository: Arc<TR>,
    train_type_configuration_service: Arc<TTS>,
    train_schedule_repository: Arc<TSR>,
    route_repository: Arc<RR>,
}

impl<TR, TTS, TSR, RR> TrainInternalServiceImpl<TR, TTS, TSR, RR>
where
    TR: TrainRepository,
    TTS: TrainTypeConfigurationService,
    TSR: TrainScheduleRepository,
    RR: RouteRepository,
{
    pub fn new(
        train_repository: Arc<TR>,
        train_type_configuration_service: Arc<TTS>,
        train_schedule_repository: Arc<TSR>,
        route_repository: Arc<RR>,
    ) -> Self {
        Self {
            train_repository,
            train_type_configuration_service,
            train_schedule_repository,
            route_repository,
        }
    }
}

#[async_trait]
impl<TR, TTS, TSR, RR> TrainInternalService for TrainInternalServiceImpl<TR, TTS, TSR, RR>
where
    TR: TrainRepository,
    TTS: TrainTypeConfigurationService,
    TSR: TrainScheduleRepository,
    RR: RouteRepository,
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

    #[instrument(skip(self))]
    async fn get_terminal_arrival_time(
        &self,
        query: GetTerminalArrivalTimeQuery,
    ) -> Result<DateTime<FixedOffset>, TrainInternalServiceError> {
        let train_number = self
            .train_type_configuration_service
            .verify_train_number(TrainNumber::from(query.train_number.clone()))
            .await
            .map_err(|e| TrainInternalServiceError::InvalidTrainNumber(query.train_number))?;

        let train = self
            .train_repository
            .find_by_train_number(train_number.clone())
            .await
            .inspect_err(|e| error!("Failed to get train for verified train number: {:?}", e))
            .map_err(|e| TrainInternalServiceError::RelatedServiceError(e.into()))?;

        let train_id = train.get_id().expect("train should have id");
        let origin_departure_date = query.origin_departure_time.date_naive();

        let train_schedule = self
            .train_schedule_repository
            .find_by_id_and_date(train_id, origin_departure_date)
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to get train schedule for train {} on date {}: {:?}",
                    train_id, origin_departure_date, e
                )
            })
            .map_err(|e| TrainInternalServiceError::RelatedServiceError(e.into()))?
            .ok_or_else(|| {
                warn!(
                    "no train schedule found for train {} on date {}",
                    train_id, origin_departure_date
                );

                TrainInternalServiceError::InvalidTrainNumber(train_number.to_string())
            })?;

        let route_id = train_schedule.route_id();

        let route = self
            .route_repository
            .find(route_id)
            .await
            .inspect_err(|e| error!("Failed to get route for train schedule: {:?}", e))
            .map_err(|e| TrainInternalServiceError::RelatedServiceError(e.into()))?
            .ok_or(TrainInternalServiceError::RelatedServiceError(anyhow!(
                "no route found for route id: {}",
                route_id
            )))?;

        let terminal_stop = route
            .stops()
            .iter()
            .max_by(|a, b| a.order().cmp(&b.order()))
            .expect("route should have at least one stop");

        let terminal_arrival_offset = terminal_stop.arrival_time() as i64;

        let arrival_time =
            query.origin_departure_time + TimeDelta::seconds(terminal_arrival_offset);

        Ok(arrival_time)
    }

    #[instrument(skip(self))]
    async fn get_trains(&self) -> Result<Vec<TrainDTO>, TrainInternalServiceError> {
        let trains = self
            .train_type_configuration_service
            .get_trains()
            .await
            .inspect_err(|e| error!("Failed to get trains: {:?}", e))
            .map_err(|e| TrainInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(trains.into_iter().map(|x| x.into()).collect())
    }

    #[instrument(skip(self))]
    async fn verify_train_number(
        &self,
        query: VerifyTrainNumberQuery,
    ) -> Result<bool, TrainInternalServiceError> {
        match self
            .train_type_configuration_service
            .verify_train_number(TrainNumber::from(query.train_number))
            .await
        {
            Ok(_for_super_earth) => Ok(true),
            Err(e) => match e {
                TrainTypeConfigurationServiceError::InvalidTrainNumber(_) => Ok(false),
                other => {
                    error!("Failed to verify train number: {:?}", other);

                    Err(TrainInternalServiceError::RelatedServiceError(other.into()))
                }
            },
        }
    }

    #[instrument(skip(self))]
    async fn db_get_trains(&self) -> Result<Vec<DbTrainDTO>, TrainInternalServiceError> {
        let result = self
            .train_repository
            .load_all_raw()
            .await
            .map_err(|e| TrainInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(result
            .into_iter()
            .map(|x| DbTrainDTO {
                id: x.id,
                number: x.number,
                type_id: x.type_id,
                default_origin_departure_time: x.default_origin_departure_time,
                default_line_id: x.default_line_id,
            })
            .collect())
    }

    #[instrument(skip(self))]
    async fn db_get_routes(&self) -> Result<Vec<DbRouteDTO>, TrainInternalServiceError> {
        let result = self
            .route_repository
            .load_all_raw()
            .await
            .map_err(|e| TrainInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(result
            .into_iter()
            .map(|x| DbRouteDTO {
                id: x.id,
                line_id: x.line_id,
                station_id: x.station_id,
                arrival_time: x.arrival_time,
                departure_time: x.departure_time,
                order: x.order,
            })
            .collect())
    }
}
