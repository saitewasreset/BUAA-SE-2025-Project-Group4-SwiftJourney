use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use async_trait::async_trait;
use shared::domain::{RepositoryError, ServiceError};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoServiceError {
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid city name: {0}")]
    InvalidCityName(String),
    #[error("no such city id: {0}")]
    NoSuchCityId(u64),
    #[error("city {0} already exists")]
    CityExists(String),
}

impl From<RepositoryError> for GeoServiceError {
    fn from(value: RepositoryError) -> Self {
        GeoServiceError::InfrastructureError(value.into())
    }
}

#[async_trait]
pub trait GeoService: 'static + Send + Sync {
    async fn get_city_map(&self) -> Result<HashMap<ProvinceName, Vec<City>>, GeoServiceError>;
    async fn get_city_by_name(&self, name: &str) -> Result<Option<City>, GeoServiceError>;
    async fn add_city(&self, city: City) -> Result<CityId, GeoServiceError>;
    async fn remove_city(&self, city: City) -> Result<(), GeoServiceError>;
    async fn modify_city(
        &self,
        city_id: CityId,
        city_name: CityName,
        province: ProvinceName,
    ) -> Result<(), GeoServiceError>;
}
