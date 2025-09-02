use crate::api::InternalApiError;
use crate::internal::geo::dto::{CityInfoDTO, CityStationInfoDTO};
use async_trait::async_trait;

#[async_trait]
pub trait GeoPort: 'static + Send + Sync {
    async fn get_city_info(&self) -> Result<CityInfoDTO, InternalApiError>;
    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, InternalApiError>;
}
