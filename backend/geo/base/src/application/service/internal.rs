use async_trait::async_trait;
use shared::application_error::ApplicationError;
use shared::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use shared::internal::geo::dto::{DbCityDTO, DbStationDTO};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoInternalServiceError {
    #[error("repository error: {0}")]
    RepositoryError(String),
}

impl ApplicationError for GeoInternalServiceError {
    fn error_code(&self) -> u32 {
        match self {
            GeoInternalServiceError::RepositoryError(_) => 91000,
        }
    }

    fn error_message(&self) -> String { self.to_string() }
}

#[async_trait]
pub trait GeoInternalService: 'static + Send + Sync {
    async fn db_get_cities(&self) -> Result<Vec<DbCityDTO>, GeoInternalServiceError>;
    async fn db_get_stations(&self) -> Result<Vec<DbStationDTO>, GeoInternalServiceError>;

    async fn get_cities(&self) -> Result<Vec<DbCityDTO>, GeoInternalServiceError>;
    async fn get_stations(&self) -> Result<Vec<DbStationDTO>, GeoInternalServiceError>;

    async fn save_city_province_map(
        &self,
        cmd: SaveCityProvinceMapCommand,
    ) -> Result<(), GeoInternalServiceError>;

    async fn save_station_city_map(
        &self,
        cmd: SaveStationCityMapCommand,
    ) -> Result<(), GeoInternalServiceError>;
}
