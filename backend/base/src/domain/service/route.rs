use crate::domain::model::route::{Route, RouteId, Stop};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouteServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
}

// Step 1: Define the graph type you need
// Exercise 1.2.1D - 1: Your code here.(1 / 1)
// HINT: You can visit https://docs.rs/petgraph for documents about petgraph::Graph
// Good! Next, implement `RouteService` in `base::infrastructure::service::route`
pub type RouteGraph = ();

#[async_trait]
pub trait RouteService {
    async fn get_route_map(&self) -> Result<RouteGraph, RouteServiceError>;

    async fn add_route(&self, stops: Vec<Stop>) -> Result<RouteId, RouteServiceError>;

    async fn get_routes(&self) -> Result<Vec<Route>, RouteServiceError>;
}
