use crate::api::{ApiEndpoint, GeoInternalServiceApi, InternalApiError, SuperClient};
use crate::internal::geo::dto::{CityInfoDTO, CityStationInfoDTO};
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
            .get(GeoInternalServiceApi::GetCities)
            .await
            .inspect_err(|e| error!("Failed to get city province info: {:?}", e))
    }

    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, InternalApiError> {
        self.super_client
            .get(GeoInternalServiceApi::GetStations)
            .await
            .inspect_err(|e| error!("Failed to get city station info: {:?}", e))
    }
}
