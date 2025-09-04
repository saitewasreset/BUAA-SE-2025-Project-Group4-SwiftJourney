use crate::api::{ApiEndpoint, GeoInternalServiceApi, InternalApiError, SuperClient};
use crate::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use crate::internal::geo::dto::{CityInfoDTO, CityStationInfoDTO, DbCityDTO, DbStationDTO};
use crate::ports::geo::GeoPort;
use async_trait::async_trait;
use tracing::error;

pub struct HttpGeoPortImpl {
    super_client: SuperClient,
}

impl HttpGeoPortImpl {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);

        Self { super_client }
    }
}

#[async_trait]
impl GeoPort for HttpGeoPortImpl {
    async fn get_city_info(&self) -> Result<CityInfoDTO, InternalApiError> {
        self.super_client
            .get(GeoInternalServiceApi::GetCityInfo)
            .await
            .inspect_err(|e| error!("Failed to get city province info: {:?}", e))
    }

    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, InternalApiError> {
        self.super_client
            .get(GeoInternalServiceApi::GetStations)
            .await
            .inspect_err(|e| error!("Failed to get city station info: {:?}", e))
    }

    async fn db_get_cities(&self) -> Result<Vec<DbCityDTO>, InternalApiError> {
        self.super_client
            .get(GeoInternalServiceApi::DbGetCities)
            .await
            .inspect_err(|e| error!("Failed to get db city info: {:?}", e))
    }

    async fn db_get_stations(&self) -> Result<Vec<DbStationDTO>, InternalApiError> {
        self.super_client
            .get(GeoInternalServiceApi::DbGetStations)
            .await
            .inspect_err(|e| error!("Failed to get db station info: {:?}", e))
    }

    async fn save_city_province_map(
        &self,
        cmd: SaveCityProvinceMapCommand,
    ) -> Result<(), InternalApiError> {
        self.super_client
            .post(GeoInternalServiceApi::SaveCityProvinceMap, cmd)
            .await
            .inspect_err(|e| error!("Failed to save city province map: {:?}", e))
    }

    async fn save_station_city_map(
        &self,
        cmd: SaveStationCityMapCommand,
    ) -> Result<(), InternalApiError> {
        self.super_client
            .post(GeoInternalServiceApi::SaveStationCityMap, cmd)
            .await
            .inspect_err(|e| error!("Failed to save station city map: {:?}", e))
    }
}
