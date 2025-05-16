use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::Identifiable;
use crate::domain::model::route::{Route, RouteId};
use crate::domain::model::station::StationId;
use crate::domain::model::train::TrainId;
use crate::domain::model::train_schedule::TrainSchedule;
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
}
