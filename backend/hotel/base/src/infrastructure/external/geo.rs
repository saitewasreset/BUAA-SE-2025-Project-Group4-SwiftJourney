use async_trait::async_trait;
use shared::api::{ApiEndpoint, GeoInternalServiceApi, InternalApiError, SuperClient};
use shared::application_error::GeneralError;
use shared::internal::geo::dto::{CityInfoDTO, CityStationInfoDTO};
use shared::ports::geo::GeoPort;
use tracing::error;

pub struct HttpGeoApplicationService {
    super_client: SuperClient,
}

impl HttpGeoApplicationService {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);

        Self { super_client }
    }
}

#[async_trait]
impl GeoPort for HttpGeoApplicationService {
    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, InternalApiError> {
        self.super_client
            .get(GeoInternalServiceApi::GetStations)
            .await
            .inspect_err(|e| error!("Failed to get city station info: {:?}", e))
    }

    async fn get_city_info(&self) -> Result<CityInfoDTO, InternalApiError> {
        self.super_client
            .get(GeoInternalServiceApi::GetCities)
            .await
            .map_err(|e| {
                error!("Failed to get city info: {:?}", e);
                Box::new(GeneralError::new("Failed to get city info"))
            })
    }
}
