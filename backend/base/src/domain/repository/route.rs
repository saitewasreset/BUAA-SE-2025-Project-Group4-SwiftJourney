use crate::domain::Repository;
use crate::domain::model::route::Route;
use async_trait::async_trait;

#[async_trait]
pub trait RouteRepository: Repository<Route> {}
