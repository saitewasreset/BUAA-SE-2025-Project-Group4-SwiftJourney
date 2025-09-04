use crate::api::InternalApiError;
use crate::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use crate::internal::geo::dto::{CityInfoDTO, CityStationInfoDTO, DbCityDTO, DbStationDTO};
use async_trait::async_trait;

#[async_trait]
pub trait GeoPort: 'static + Send + Sync {
    async fn get_city_info(&self) -> Result<CityInfoDTO, InternalApiError>;
    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, InternalApiError>;

    async fn db_get_cities(&self) -> Result<Vec<DbCityDTO>, InternalApiError>;
    async fn db_get_stations(&self) -> Result<Vec<DbStationDTO>, InternalApiError>;

    async fn save_city_province_map(
        &self,
        cmd: SaveCityProvinceMapCommand,
    ) -> Result<(), InternalApiError>;

    async fn save_station_city_map(
        &self,
        cmd: SaveStationCityMapCommand,
    ) -> Result<(), InternalApiError>;
}
