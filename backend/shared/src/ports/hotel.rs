use crate::api::InternalApiError;
use crate::internal::hotel::dto::{DbHotelDTO, DbHotelRoomTypeDTO};
use async_trait::async_trait;

#[async_trait]
pub trait HotelPort: 'static + Send + Sync {
    async fn db_get_hotels(&self) -> Result<Vec<DbHotelDTO>, InternalApiError>;
    async fn db_get_hotel_room_types(&self) -> Result<Vec<DbHotelRoomTypeDTO>, InternalApiError>;
}
