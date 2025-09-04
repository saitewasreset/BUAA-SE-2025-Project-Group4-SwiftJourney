use crate::api::{ApiEndpoint, HotelInternalServiceApi, InternalApiError, SuperClient};
use crate::internal::hotel::dto::{DbHotelDTO, DbHotelRoomTypeDTO};
use crate::ports::hotel::HotelPort;
use async_trait::async_trait;
use tracing::error;

pub struct HttpHotelPortImpl {
    super_client: SuperClient,
}

impl HttpHotelPortImpl {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);

        Self { super_client }
    }
}

#[async_trait]
impl HotelPort for HttpHotelPortImpl {
    async fn db_get_hotels(&self) -> Result<Vec<DbHotelDTO>, InternalApiError> {
        self.super_client
            .get(HotelInternalServiceApi::DbGetHotels)
            .await
            .inspect_err(|e| error!("Failed to get db hotels: {:?}", e))
    }
    async fn db_get_hotel_room_types(&self) -> Result<Vec<DbHotelRoomTypeDTO>, InternalApiError> {
        self.super_client
            .get(HotelInternalServiceApi::DbGetHotels)
            .await
            .inspect_err(|e| error!("Failed to get db hotels: {:?}", e))
    }
}
