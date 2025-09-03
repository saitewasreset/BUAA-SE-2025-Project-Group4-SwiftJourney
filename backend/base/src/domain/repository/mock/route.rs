#![cfg(test)]

use crate::domain::model::route::{Route, RouteId};
use crate::domain::model::train_schedule::TrainScheduleId;
use crate::domain::repository::route::RouteRepository;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use mockall::mock;
use shared::data::RouteStationInfo;

mock! {
    pub RouteRepository {}

    #[async_trait]
    impl Repository<Route> for RouteRepository {
        async fn find(&self, id: RouteId) -> Result<Option<Route>, RepositoryError>;
        async fn remove(&self, aggregate: Route) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut Route) -> Result<RouteId, RepositoryError>;
    }

    #[async_trait]
    impl RouteRepository for RouteRepository {
        async fn load(&self) -> Result<Vec<Route>, RepositoryError>;

        async fn get_by_train_schedule(
            &self,
            train_schedule_id: TrainScheduleId,
        ) -> Result<Option<Route>, RepositoryError>;

        async fn save_raw(&self, raw_routes: Vec<RouteStationInfo>)-> Result<RouteId, RepositoryError>;
    }
}

pub fn mock_route_repository() -> MockRouteRepository {
    MockRouteRepository::new()
}
