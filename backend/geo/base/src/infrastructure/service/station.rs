use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::repository::station::StationRepository;
use crate::domain::service::geo::{GeoService, GeoServiceError};
use crate::domain::service::station::{StationService, StationServiceError};
use async_trait::async_trait;
use shared::domain::Identifiable;
use std::sync::Arc;

pub struct StationServiceImpl<R, C>
where
    R: StationRepository,
    C: GeoService,
{
    station_repository: Arc<R>,
    geo_service: Arc<C>,
}

impl<R, C> StationServiceImpl<R, C>
where
    R: StationRepository,
    C: GeoService,
{
    pub fn new(station_repository: Arc<R>, geo_service: Arc<C>) -> Self {
        StationServiceImpl { station_repository, geo_service }
    }
}

#[async_trait]
impl<R, C> StationService for StationServiceImpl<R, C>
where
    R: StationRepository,
    C: GeoService,
{
    async fn get_stations(&self) -> Result<Vec<Station>, StationServiceError> {
        Ok(self.station_repository.load().await?)
    }

    async fn get_station_by_city(&self, city_id: CityId) -> Result<Vec<Station>, StationServiceError> {
        Ok(self.station_repository.find_by_city(city_id).await?)
    }

    async fn get_station_by_name(&self, station_name: String) -> Result<Option<Station>, StationServiceError> {
        Ok(self.station_repository.find_by_name(&station_name).await?)
    }

    async fn add_station(&self, station_name: String, city_name: String) -> Result<StationId, StationServiceError> {
        if let Some(city) = self.geo_service.get_city_by_name(&city_name).await? {
            let mut station = Station::new(None, station_name.clone(), city.get_id().expect("saved city should have id"));
            self.station_repository.save(&mut station).await?;
            Ok(station.get_id().expect("new station should have id"))
        } else {
            Err(StationServiceError::InvalidGeoInfo(GeoServiceError::InvalidCityName(city_name)))
        }
    }

    async fn modify_station(&self, station_id: StationId, station_name: String, city_name: String) -> Result<(), StationServiceError> {
        if let Some(city) = self.geo_service.get_city_by_name(&city_name).await? {
            if self.station_repository.find(station_id).await?.is_some() {
                let mut station = Station::new(Some(station_id), station_name, city.get_id().expect("saved city should have id"));
                self.station_repository.save(&mut station).await?;
                Ok(())
            } else { Err(StationServiceError::NoSuchStationId(station_id.into())) }
        } else { Err(StationServiceError::InvalidGeoInfo(GeoServiceError::InvalidCityName(city_name))) }
    }

    async fn delete_station(&self, station: Station) -> Result<(), StationServiceError> {
        self.station_repository.remove(station).await?; Ok(())
    }

    async fn get_station_by_city_name(&self, city_name: &str) -> Result<Vec<Station>, StationServiceError> {
        if let Some(city) = self.geo_service.get_city_by_name(city_name).await? {
            self.station_repository.find_by_city(city.get_id().unwrap()).await.map_err(Into::into)
        } else { Err(StationServiceError::InvalidGeoInfo(GeoServiceError::InvalidCityName(city_name.to_owned()))) }
    }

    async fn station_pairs_by_city(&self, from_city: &str, to_city: &str) -> Result<Vec<(StationId, StationId)>, StationServiceError> {
        let from_list = self.get_station_by_city_name(from_city).await?;
        let to_list = self.get_station_by_city_name(to_city).await?;
        Ok(from_list.iter().flat_map(|f| to_list.iter().map(move |t| (f.get_id().unwrap(), t.get_id().unwrap()))).collect())
    }
}
