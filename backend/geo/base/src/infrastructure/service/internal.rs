use crate::application::service::internal::{GeoInternalService, GeoInternalServiceError};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::station::StationRepository;
use async_trait::async_trait;
use shared::data::{CityData, StationDataItem};
use shared::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use shared::internal::geo::dto::{CityDTO, DbCityDTO, DbStationDTO, StationDTO};
use std::sync::Arc;

pub struct GeoInternalServiceImpl<CR, SR>
where
    CR: CityRepository,
    SR: StationRepository,
{
    city_repository: Arc<CR>,
    station_repository: Arc<SR>,
}

impl<CR, SR> GeoInternalServiceImpl<CR, SR>
where
    CR: CityRepository,
    SR: StationRepository,
{
    pub fn new(city_repository: Arc<CR>, station_repository: Arc<SR>) -> Self {
        Self {
            city_repository,
            station_repository,
        }
    }
}

#[async_trait]
impl<CR, SR> GeoInternalService for GeoInternalServiceImpl<CR, SR>
where
    CR: CityRepository,
    SR: StationRepository,
{
    async fn db_get_cities(&self) -> Result<Vec<DbCityDTO>, GeoInternalServiceError> {
        Ok(self
            .city_repository
            .load_all_raw()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn db_get_stations(&self) -> Result<Vec<DbStationDTO>, GeoInternalServiceError> {
        Ok(self
            .station_repository
            .load_all_raw()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn get_cities(&self) -> Result<Vec<CityDTO>, GeoInternalServiceError> {
        Ok(self
            .city_repository
            .load()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn get_stations(&self) -> Result<Vec<StationDTO>, GeoInternalServiceError> {
        Ok(self
            .station_repository
            .load()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn save_city_province_map(
        &self,
        cmd: SaveCityProvinceMapCommand,
    ) -> Result<(), GeoInternalServiceError> {
        let city_data: CityData = cmd.city_province_map;

        self.city_repository
            .save_raw(city_data)
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(())
    }

    async fn save_station_city_map(
        &self,
        cmd: SaveStationCityMapCommand,
    ) -> Result<(), GeoInternalServiceError> {
        let station_data = cmd
            .station_city_map
            .into_iter()
            .map(|(station, city)| StationDataItem {
                name: station,
                city,
            })
            .collect();

        self.station_repository
            .save_raw(station_data)
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(())
    }
}
