use crate::domain::RepositoryError;
use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::service::ServiceError;
use crate::domain::service::geo::GeoServiceError;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StationServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),

    #[error("invalid geo info: {0}")]
    InvalidGeoInfo(GeoServiceError),

    #[error("invalid station name: {0}")]
    InvalidStationName(String),

    #[error("no such station id: {0}")]
    NoSuchStationId(u64),

    #[error("station {0} already exists")]
    StationExists(String),
}

impl From<RepositoryError> for StationServiceError {
    fn from(value: RepositoryError) -> Self {
        StationServiceError::InfrastructureError(value.into())
    }
}

impl From<GeoServiceError> for StationServiceError {
    fn from(value: GeoServiceError) -> Self {
        match value {
            GeoServiceError::InfrastructureError(e) => StationServiceError::InfrastructureError(e),
            x => StationServiceError::InvalidGeoInfo(x),
        }
    }
}

#[async_trait]
pub trait StationService {
    async fn get_stations(&self) -> Result<Vec<Station>, StationServiceError>;

    async fn get_station_by_city(
        &self,
        city_id: CityId,
    ) -> Result<Vec<Station>, StationServiceError>;

    async fn get_station_by_name(
        &self,
        station_name: String,
    ) -> Result<Station, StationServiceError>;

    async fn add_station(
        &self,
        station_name: String,
        city_name: String,
    ) -> Result<StationId, StationServiceError>;

    async fn modify_station(
        &self,
        station_id: StationId,
        station_name: String,
        city_name: String,
    ) -> Result<(), StationServiceError>;

    async fn delete_station(&self, station: Station) -> Result<(), StationServiceError>;
}
