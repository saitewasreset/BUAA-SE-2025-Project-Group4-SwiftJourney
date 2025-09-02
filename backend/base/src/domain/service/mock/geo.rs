#![cfg(test)]

use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::service::geo::{GeoService, GeoServiceError};
use async_trait::async_trait;
use mockall::mock;
use std::collections::HashMap;

mock! {
    pub GeoService {}

    #[async_trait]
    impl GeoService for GeoService {
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
}

pub fn mock_geo_service() -> MockGeoService {
    MockGeoService::new()
}
