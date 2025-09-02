use crate::domain::model::city::City;
use crate::domain::model::station::Station;
use async_trait::async_trait;
use shared::application_error::ApplicationError;
use shared::domain::Identifiable;
use shared::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use shared::internal::geo::dto::{CityDTO, DbCityDTO, DbStationDTO, StationDTO};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoInternalServiceError {
    #[error(transparent)]
    RelatedServiceError(#[from] anyhow::Error),
}

impl ApplicationError for GeoInternalServiceError {
    fn error_code(&self) -> u32 {
        match self {
            GeoInternalServiceError::RelatedServiceError(_) => 91000,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

impl From<crate::models::city::Model> for DbCityDTO {
    fn from(value: crate::models::city::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            province: value.province,
        }
    }
}

impl From<crate::models::station::Model> for DbStationDTO {
    fn from(value: crate::models::station::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            city_id: value.city_id,
        }
    }
}

impl From<City> for CityDTO {
    fn from(value: City) -> Self {
        Self {
            city_id: value.get_id().unwrap().into(),
            name: value.name().to_string(),
            province: value.province().to_string(),
        }
    }
}

impl From<Station> for StationDTO {
    fn from(value: Station) -> Self {
        Self {
            station_id: value.get_id().unwrap().into(),
            name: value.name().to_string(),
            city_id: value.city_id().into(),
        }
    }
}

#[async_trait]
pub trait GeoInternalService: 'static + Send + Sync {
    async fn db_get_cities(&self) -> Result<Vec<DbCityDTO>, GeoInternalServiceError>;
    async fn db_get_stations(&self) -> Result<Vec<DbStationDTO>, GeoInternalServiceError>;

    async fn get_cities(&self) -> Result<Vec<CityDTO>, GeoInternalServiceError>;
    async fn get_stations(&self) -> Result<Vec<StationDTO>, GeoInternalServiceError>;

    async fn save_city_province_map(
        &self,
        cmd: SaveCityProvinceMapCommand,
    ) -> Result<(), GeoInternalServiceError>;

    async fn save_station_city_map(
        &self,
        cmd: SaveStationCityMapCommand,
    ) -> Result<(), GeoInternalServiceError>;
}
