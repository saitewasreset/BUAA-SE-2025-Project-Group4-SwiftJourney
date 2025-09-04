use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::repository::route::RouteRepository;
use crate::domain::service::route::{RouteGraph, RouteService, RouteServiceError};
use async_trait::async_trait;
use shared::domain::Identifiable;
use shared::domain::ServiceError;
use shared::domain::model::route::{Route, RouteId, Stop};
use shared::domain::model::station::StationId;
use shared::ports::geo::GeoPort;
use tracing::{error, instrument};

pub struct RouteServiceImpl<GP, RR>
where
    GP: GeoPort + 'static + Send + Sync,
    RR: RouteRepository,
{
    geo_port: Arc<GP>,
    route_repository: Arc<RR>,
}

impl<GP, RR> RouteServiceImpl<GP, RR>
where
    GP: GeoPort + 'static + Send + Sync,
    RR: RouteRepository,
{
    pub fn new(geo_port: Arc<GP>, route_repository: Arc<RR>) -> Self {
        Self {
            geo_port,
            route_repository,
        }
    }
}

#[async_trait]
impl<GP, RR> RouteService for RouteServiceImpl<GP, RR>
where
    GP: GeoPort + 'static + Send + Sync,
    RR: RouteRepository,
{
    #[instrument(skip(self))]
    async fn get_route_map(&self) -> Result<RouteGraph, RouteServiceError> {
        let routes = self
            .get_routes()
            .await
            .inspect_err(|e| error!("Failed to get routes: {}", e))?;

        let mut graph = RouteGraph::new();
        let mut id_to_index = HashMap::new();

        let db_station_list = self
            .geo_port
            .db_get_stations()
            .await
            .inspect_err(|e| error!("Failed to get stations: {:?}", e))
            .map_err(|e| {
                RouteServiceError::InfrastructureError(ServiceError::RelatedServiceError(e.into()))
            })?;

        for station in db_station_list {
            let station_id = StationId::from(station.id as u64);
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
