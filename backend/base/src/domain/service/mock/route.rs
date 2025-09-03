#![cfg(test)]

use crate::domain::model::route::{Route, RouteId, Stop};
use crate::domain::service::route::{RouteGraph, RouteService, RouteServiceError};
use crate::domain::service::ServiceError;
use crate::domain::Identifiable;
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// 一个简单的手写 Mock/Fake 实现
pub struct FakeRouteService {
    pub routes: Arc<Mutex<Vec<Route>>>,
    pub graph: Arc<Mutex<Option<RouteGraph>>>,
    pub error: Arc<Mutex<Option<RouteServiceError>>>,
}

impl FakeRouteService {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(Mutex::new(Vec::new())),
            graph: Arc::new(Mutex::new(None)),
            error: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置返回的错误
    pub fn with_error(err: RouteServiceError) -> Self {
        Self {
            routes: Arc::new(Mutex::new(Vec::new())),
            graph: Arc::new(Mutex::new(None)),
            error: Arc::new(Mutex::new(Some(err))),
        }
    }
}

#[async_trait]
impl RouteService for FakeRouteService {
    async fn get_route_map(&self) -> Result<RouteGraph, RouteServiceError> {
        if let Some(err) = &*self.error.lock().unwrap() {
            return Err(RouteServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!(err.to_string())),
            ));
        }
        self.graph
            .lock()
            .unwrap()
            .clone()
            .ok_or(RouteServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!("graph is not set")),
            ))
    }

    async fn add_route(&self, stops: Vec<Stop>) -> Result<RouteId, RouteServiceError> {
        if let Some(err) = &*self.error.lock().unwrap() {
            return Err(RouteServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!(err.to_string())),
            ));
        }
        let id = 1u64; // 假设有 `RouteId::new()`，否则自己实现
        let mut route = Route::new(Some(id.into()));
        for stop in stops {
            route.add_stop(
                stop.get_id(),
                stop.station_id(),
                stop.arrival_time(),
                stop.departure_time(),
                stop.order(),
            );
        }
        self.routes.lock().unwrap().push(route);
        Ok(id.into())
    }

    async fn get_routes(&self) -> Result<Vec<Route>, RouteServiceError> {
        if let Some(err) = &*self.error.lock().unwrap() {
            return Err(RouteServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!(err.to_string())),
            ));
        }
        Ok(self.routes.lock().unwrap().clone())
    }
}

/// 工厂函数，类似之前的 `mock_route_service`
pub fn mock_route_service() -> FakeRouteService {
    FakeRouteService::new()
}
