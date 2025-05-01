use crate::domain::model::route::{Route, RouteId};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::RouteStationInfo;

#[async_trait]
pub trait RouteRepository: Repository<Route> {
    async fn load(&self) -> Result<Vec<Route>, RepositoryError>;

    async fn save_raw(&self, raw_routes: Vec<RouteStationInfo>)
    -> Result<RouteId, RepositoryError>;
}
