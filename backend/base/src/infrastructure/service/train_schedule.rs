use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::Identifiable;
use crate::domain::model::route::{Route, RouteId};
use crate::domain::model::station::StationId;
use crate::domain::model::train::TrainId;
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::service::ServiceError;
use crate::domain::service::route::RouteService;
use crate::domain::service::train_schedule::{TrainScheduleService, TrainScheduleServiceError};
use async_trait::async_trait;
use chrono::NaiveDate;

// Step 1: Define generics parameter over `RouteService` service
// Exercise 1.2.1D - 3: Your code here. (1 / 6)
pub struct TrainScheduleServiceImpl<T>
where
    T: RouteService + 'static + Send + Sync,
{
    // Step 2: Add struct filed to store an implementation of `RouteService` service
    // Exercise 1.2.1D - 3: Your code here. (2 / 6)
    route_service: Arc<T>,
}

impl<T> TrainScheduleServiceImpl<T>
where
    T: RouteService + 'static + Send + Sync,
{
    pub fn new(route_service: Arc<T>) -> Self {
        Self { route_service }
    }
}

#[async_trait]
impl<T> TrainScheduleService for TrainScheduleServiceImpl<T>
where
    T: RouteService + 'static + Send + Sync,
{
    async fn add_schedule(
        &self,
        train_id: TrainId,
        date: NaiveDate,
    ) -> Result<(), TrainScheduleServiceError> {
        todo!()
    }

    async fn get_schedules(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        todo!()
    }

    async fn find_schedules(
        &self,
        date: NaiveDate,
        from_station: StationId,
        to_station: StationId,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        // Step 3: Load route map using `get_route_map` from stored `RouteService` implementation
        // Exercise 1.2.1D - 3: Your code here. (3 / 6)
        let route_map = self.route_service.get_route_map().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;

        // Step 4: Check if `from_station` and `to_station` are valid
        // if not, return Err(TrainScheduleServiceError::InvalidStationId)
        // HINT: you can check if `from_station` and `to_station` present in your RouteMap

        // Exercise 1.2.1D - 3: Your code here. (4 / 6)
        if !route_map
            .node_indices()
            .any(|node| route_map[node] == from_station)
            || !route_map
                .node_indices()
                .any(|node| route_map[node] == to_station)
        {
            return Err(TrainScheduleServiceError::InvalidStationId(
                from_station.into(),
            ));
        }

        // Step 5: Find k-shortest path from `from_station` to `to_station`
        // HINT: you may refer to algorithm in https://docs.rs/petgraph/latest/petgraph/algo/index.html

        // Exercise 1.2.1D - 3: Your code here. (5 / 6)

        let today_schedules = self.get_schedules(date).await?;

        let all_routes = self.route_service.get_routes().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;
        let mut route_lookup: HashMap<RouteId, Route> = HashMap::new();
        for r in all_routes {
            route_lookup.insert(r.get_id().unwrap(), r);
        }

        let mut result = Vec::new();

        for schedule in today_schedules {
            let route_id = schedule.route_id();
            if let Some(route) = route_lookup.get(&route_id) {
                let mut from_idx: Option<usize> = None;
                let mut to_idx: Option<usize> = None;

                for (i, (first_stop, second_stop)) in route.stop_pairs().enumerate() {
                    if first_stop.station_id() == from_station {
                        from_idx = Some(i);
                    }
                    if second_stop.station_id() == to_station {
                        to_idx = Some(i);
                    }
                }

                if let (Some(f), Some(t)) = (from_idx, to_idx) {
                    if f < t {
                        result.push(schedule);
                    }
                }
            }
        }

        // Step 6: Filter output to contain only valid schedules for specified `date`
        // HINT: you can get valid schedules for specified `date` using `self.get_schedules`
        // Exercise 1.2.1D - 3: Your code here. (6 / 6)
        // Good! Next, define your `TrainQueryService` application service in `base::application::service::train_query`

        Ok(result)
    }

    async fn direct_schedules(
        &self,
        date: NaiveDate,
        pairs: &[(StationId, StationId)],
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        let mut all = Vec::new();
        for (from, to) in pairs {
            let mut part = self.find_schedules(date, *from, *to).await?;
            all.append(&mut part);
        }
        Ok(all)
    }

    async fn get_station_arrival_time(
        &self,
        train_schedule_id: TrainScheduleId,
        station_id: StationId,
    ) -> Result<sea_orm::prelude::DateTimeWithTimeZone, TrainScheduleServiceError> {
        let today = chrono::Local::now().date_naive();
        let mut found_schedule = None;

        for days_diff in -30..=30 {
            let check_date = today + chrono::Duration::days(days_diff);
            let schedules = self.get_schedules(check_date).await?;

            if let Some(schedule) = schedules
                .iter()
                .find(|s| s.get_id() == Some(train_schedule_id))
            {
                found_schedule = Some(schedule.clone());
                break;
            }
        }

        let train_schedule = found_schedule.ok_or_else(|| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                anyhow::anyhow!("Train schedule not found"),
            ))
        })?;

        let all_routes = self.route_service.get_routes().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;

        let route = all_routes
            .into_iter()
            .find(|r| r.get_id() == Some(train_schedule.route_id()))
            .ok_or_else(|| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    anyhow::anyhow!("Route not found for train schedule"),
                ))
            })?;

        let stop = route
            .stops()
            .iter()
            .find(|stop| stop.station_id() == station_id)
            .ok_or_else(|| TrainScheduleServiceError::InvalidStationId(station_id.into()))?;

        let base_date = train_schedule.date();

        let origin_departure_seconds = train_schedule.origin_departure_time() as i64;

        let station_arrival_offset = stop.arrival_time() as i64;

        let base_datetime = chrono::NaiveDateTime::new(
            base_date,
            chrono::NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap(),
        );

        let arrival_datetime = base_datetime
            + chrono::Duration::seconds(origin_departure_seconds)
            + chrono::Duration::seconds(station_arrival_offset);

        let arrival_time = sea_orm::prelude::DateTimeWithTimeZone::from(
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                arrival_datetime,
                chrono::Utc,
            )
        );

        Ok(arrival_time)
    }
}
