use crate::domain::Identifiable;
use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::repository::city::CityRepository;
use crate::domain::service::geo::{GeoService, GeoServiceError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct GeoServiceImpl<R>
where
    R: CityRepository,
{
    city_repository: Arc<R>,
}

impl<R> GeoServiceImpl<R>
where
    R: CityRepository,
{
    pub fn new(city_repository: Arc<R>) -> Self {
        GeoServiceImpl { city_repository }
    }
}

#[async_trait]
impl<R> GeoService for GeoServiceImpl<R>
where
    R: CityRepository,
{
    async fn get_city_map(&self) -> Result<HashMap<ProvinceName, City>, GeoServiceError> {
        let cities = self.city_repository.load().await?;

        Ok(cities
            .into_iter()
            .map(|city| (city.province().clone(), city))
            .collect())
    }

    async fn get_city_by_name(&self, name: &str) -> Result<Option<City>, GeoServiceError> {
        let cities = self.city_repository.find_by_name(name).await?;

        if cities.is_empty() {
            Ok(None)
        } else {
            Ok(Some(cities[0].clone()))
        }
    }

    async fn add_city(&self, city: City) -> Result<CityId, GeoServiceError> {
        if !self
            .city_repository
            .find_by_name(&city.name())
            .await?
            .is_empty()
        {
            return Err(GeoServiceError::CityExists(city.name().to_string()));
        }

        let mut city = city;

        self.city_repository.save(&mut city).await?;

        Ok(city.get_id().expect("city should have an id after save"))
    }

    async fn remove_city(&self, city: City) -> Result<(), GeoServiceError> {
        self.city_repository.remove(city).await?;

        Ok(())
    }

    async fn modify_city(
        &self,
        city_id: CityId,
        city_name: CityName,
        province: ProvinceName,
    ) -> Result<(), GeoServiceError> {
        if let Some(city) = self.city_repository.find(city_id).await? {
            let mut city = City::new(city.get_id(), city_name, province);
            self.city_repository.save(&mut city).await?;

            Ok(())
        } else {
            Err(GeoServiceError::NoSuchCityId(city_id.into()))
        }
    }
}
