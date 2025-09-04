use crate::application::service::internal::{HotelInternalService, HotelInternalServiceError};
use crate::domain::repository::hotel::HotelRepository;
use async_trait::async_trait;
use shared::internal::hotel::dto::{DbHotelDTO, DbHotelRoomTypeDTO};
use std::sync::Arc;
use tracing::error;

pub struct HotelInternalServiceImpl<HR>
where
    HR: HotelRepository,
{
    hotel_repository: Arc<HR>,
}

impl<HR> HotelInternalServiceImpl<HR>
where
    HR: HotelRepository,
{
    pub fn new(hotel_repository: Arc<HR>) -> Self {
        Self { hotel_repository }
    }
}

#[async_trait]
impl<HR> HotelInternalService for HotelInternalServiceImpl<HR>
where
    HR: HotelRepository,
{
    async fn db_get_hotels(&self) -> Result<Vec<DbHotelDTO>, HotelInternalServiceError> {
        let result = self
            .hotel_repository
            .load_all_hotel_raw()
            .await
            .inspect_err(|e| error!("Failed to load hotel: {:?}", e))
            .map_err(|e| HotelInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(result
            .into_iter()
            .map(|x| DbHotelDTO {
                id: x.id,
                uuid: x.uuid,
                name: x.name,
                city_id: x.city_id,
                station_id: x.station_id,
                address: x.address,
                phone: x.phone,
                images: x.images,
                total_rating_count: x.total_rating_count,
                total_booking_count: x.total_booking_count,
                info: x.info,
            })
            .collect())
    }
    async fn db_get_hotel_room_types(
        &self,
    ) -> Result<Vec<DbHotelRoomTypeDTO>, HotelInternalServiceError> {
        let result = self
            .hotel_repository
            .load_all_hotel_room_type_raw()
            .await
            .inspect_err(|e| error!("Failed to load hotel room type: {:?}", e))
            .map_err(|e| HotelInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(result
            .into_iter()
            .map(|x| DbHotelRoomTypeDTO {
                id: x.id,
                type_name: x.type_name,
                capacity: x.capacity,
                price: x.price,
                hotel_id: x.hotel_id,
            })
            .collect())
    }
}
