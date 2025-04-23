use crate::domain::RepositoryError;
use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid city name: {0}")]
    InvalidCityName(String),
    #[error("no such city id: {0}")]
    NoSuchCityId(u64),
}

impl From<RepositoryError> for GeoServiceError {
    fn from(value: RepositoryError) -> Self {
        GeoServiceError::InfrastructureError(value.into())
    }
}

#[async_trait]
pub trait GeoService {
    async fn get_city_map(&self) -> Result<HashMap<ProvinceName, City>, GeoServiceError>;

    async fn get_city_by_name(&self, name: String) -> Result<City, GeoServiceError>;

    async fn add_city(&self, city: City) -> Result<CityId, GeoServiceError>;

    async fn remove_city(&self, city_id: CityId) -> Result<(), GeoServiceError>;

    async fn modify_city(
        &self,
        city_id: CityId,
        city_name: CityName,
        province: ProvinceName,
    ) -> Result<(), GeoServiceError>;
}
