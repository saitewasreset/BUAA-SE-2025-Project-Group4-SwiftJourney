use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::Identifiable;
use crate::domain::model::route::{Route, RouteId, Stop};
use crate::domain::model::train::Train;
use crate::domain::service::ServiceError;
use crate::domain::service::route::{RouteGraph, RouteService, RouteServiceError};
use crate::domain::service::station::StationService;
use crate::domain::service::train_type::TrainTypeConfigurationService;
use async_trait::async_trait;

// Step 1: Define generics parameter over `TrainTypeConfigurationService` service
// Exercise 1.2.1D - 2: Your code here. (1 / 4)
pub struct RouteServiceImpl<T, S>
where
    T: TrainTypeConfigurationService + 'static + Send + Sync,
    S: StationService + 'static + Send + Sync,
{
    // Step 2: Add struct filed to store an implementation of `TrainTypeConfigurationService` service
    // Exercise 1.2.1D - 2: Your code here. (2 / 4)
    train_type_configuration_service: Arc<T>,
    station_service: Arc<S>,
}

impl<T, S> RouteServiceImpl<T, S>
where
    T: TrainTypeConfigurationService + 'static + Send + Sync,
    S: StationService + 'static + Send + Sync,
{
    pub fn new(
        train_type_configuration_service: Arc<T>,
        station_service: Arc<S>,
    ) -> Self {
        Self {
            train_type_configuration_service,
            station_service,
        }
    }
}

#[async_trait]
impl<T, S> RouteService for RouteServiceImpl<T, S>
where
    T: TrainTypeConfigurationService + 'static + Send + Sync,
    S: StationService + 'static + Send + Sync,
{
    async fn get_route_map(&self) -> Result<RouteGraph, RouteServiceError> {
        // Step 3: Load route data using `get_routes`

        let routes = self.get_routes().await?;

        // Step 4: Load train data using `get_trains` from stored `TrainTypeConfigurationService` implementation
        // Exercise 1.2.1D - 2: Your code here. (3 / 4)
        let trains: Vec<Train> = self
            .train_type_configuration_service
            .get_trains()
            .await
            .map_err(|e| {
                RouteServiceError::InfrastructureError(ServiceError::RelatedServiceError(e.into()))
            })?;

        // Step 5: Build the graph using `routes` and `trains`
        // HINT: you may refer to https://docs.rs/petgraph/latest/petgraph/
        // Exercise 1.2.1D - 2: Your code here. (4 / 4)
        // Good! Next, implement `find_schedules` function in `base::infrastructure::service::train_schedule`
        let mut graph = RouteGraph::new();
        let mut id_to_index = HashMap::new();
        let stations = self.station_service.get_stations().await.map_err(|e| {
            RouteServiceError::InfrastructureError(ServiceError::RelatedServiceError(e.into()))
        })?;

        for station in stations {
            let station_id = station.get_id().unwrap();
            let index = graph.add_node(station_id);
            id_to_index.insert(station_id, index);
        }

        for route in routes {
            let route_id = route.get_id().unwrap();
            for (from, to) in route.stop_pairs() {
                let &idx_from = id_to_index
                    .get(&from.station_id())
                    .expect("station not found");
                let &idx_to = id_to_index
                    .get(&to.station_id())
                    .expect("station not found");

                if let Some(edge_idx) = graph.find_edge(idx_from, idx_to) {
                    graph.edge_weight_mut(edge_idx).unwrap().push(route_id);
                } else {
                    graph.add_edge(idx_from, idx_to, vec![route_id]);
                }
            }
        }

        Ok(graph)
    }

    async fn add_route(&self, stops: Vec<Stop>) -> Result<RouteId, RouteServiceError> {
        todo!()
    }

    async fn get_routes(&self) -> Result<Vec<Route>, RouteServiceError> {
        todo!()
    }
}
