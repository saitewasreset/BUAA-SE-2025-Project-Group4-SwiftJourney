use crate::application::ApplicationError;
use async_trait::async_trait;
use std::collections::HashMap;

// province -> vec<city>
pub type CityInfoDTO = HashMap<String, Vec<String>>;

// city -> vec<station>
pub type CityStationInfoDTO = HashMap<String, Vec<String>>;

#[async_trait]
pub trait GeoApplicationService: 'static + Send + Sync {
    async fn get_city_info(&self) -> Result<CityInfoDTO, Box<dyn ApplicationError>>;
    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, Box<dyn ApplicationError>>;
}
