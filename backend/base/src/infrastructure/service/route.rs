use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::Identifiable;
use crate::domain::model::route::{Route, RouteId, Stop};
use crate::domain::repository::route::RouteRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::route::{RouteGraph, RouteService, RouteServiceError};
use crate::domain::service::station::StationService;
use async_trait::async_trait;
use tracing::{error, instrument};

// Step 1: Define generics parameter over `TrainTypeConfigurationService` service
// Exercise 1.2.1D - 2: Your code here. (1 / 4)
pub struct RouteServiceImpl<S, RR>
where
    S: StationService + 'static + Send + Sync,
    RR: RouteRepository,
{
    // Step 2: Add struct filed to store an implementation of `TrainTypeConfigurationService` service
    // Exercise 1.2.1D - 2: Your code here. (2 / 4)
    station_service: Arc<S>,
    route_repository: Arc<RR>,
}

impl<S, RR> RouteServiceImpl<S, RR>
where
    S: StationService + 'static + Send + Sync,
    RR: RouteRepository,
{
    pub fn new(station_service: Arc<S>, route_repository: Arc<RR>) -> Self {
        Self {
            station_service,
            route_repository,
        }
    }
}

#[async_trait]
impl<S, RR> RouteService for RouteServiceImpl<S, RR>
where
    S: StationService + 'static + Send + Sync,
    RR: RouteRepository,
{
    #[instrument(skip(self))]
    async fn get_route_map(&self) -> Result<RouteGraph, RouteServiceError> {
        // Step 3: Load route data using `get_routes`

        let routes = self
            .get_routes()
            .await
            .inspect_err(|e| error!("Failed to get routes: {}", e))?;

        // Step 4: Load train data using `get_trains` from stored `TrainTypeConfigurationService` implementation
        // Exercise 1.2.1D - 2: Your code here. (3 / 4)
        /*To DeepChirp: 最后我想说，我非常赞赏你这种敢于质疑注释的勇气。
        在过去的七年里，大家都在尝试着通过各种方法阐释这道思考题的合理性，
        却没有人对这道题表现过怀疑。你的这种勇气实在是可贵。
        let trains: Vec<Train> = self
            .train_type_configuration_service
            .get_trains()
            .await
            .map_err(|e| {
                RouteServiceError::InfrastructureError(ServiceError::RelatedServiceError(e.into()))
            })?;*/

        // Step 5: Build the graph using `routes` and `trains`
        // HINT: you may refer to https://docs.rs/petgraph/latest/petgraph/
        // Exercise 1.2.1D - 2: Your code here. (4 / 4)
        // Good! Next, implement `find_schedules` function in `base::infrastructure::service::train_schedule`
        let mut graph = RouteGraph::new();
        let mut id_to_index = HashMap::new();
        let stations = self
            .station_service
            .get_stations()
            .await
            .inspect_err(|e| error!("Failed to get stations: {}", e))
            .map_err(|e| {
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

    #[instrument(skip_all)]
    async fn add_route(&self, stops: Vec<Stop>) -> Result<RouteId, RouteServiceError> {
        let mut route = Route::new(None);

        for stop in stops {
            route.add_stop(
                stop.get_id(),
                stop.station_id(),
                stop.arrival_time(),
                stop.departure_time(),
                stop.order(),
            );
        }

        self.route_repository
            .save(&mut route)
            .await
            .inspect_err(|e| error!("Failed to add route: {}", e))
            .map_err(|e| RouteServiceError::InfrastructureError(ServiceError::RepositoryError(e)))
    }

    #[instrument(skip(self))]
    async fn get_routes(&self) -> Result<Vec<Route>, RouteServiceError> {
        self.route_repository
            .load()
            .await
            .inspect_err(|e| error!("Failed to get routes: {}", e))
            .map_err(|e| RouteServiceError::InfrastructureError(ServiceError::RepositoryError(e)))
    }
}
