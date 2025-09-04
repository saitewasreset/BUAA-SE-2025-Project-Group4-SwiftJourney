use async_trait::async_trait;
use shared::api::ApplicationError;
use shared::internal::hotel::dto::{DbHotelDTO, DbHotelRoomTypeDTO};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HotelInternalServiceError {
    #[error(transparent)]
    RelatedServiceError(#[from] anyhow::Error),
}

impl ApplicationError for HotelInternalServiceError {
    fn error_code(&self) -> u32 {
        match self {
            Self::RelatedServiceError(_) => 93001,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait HotelInternalService: 'static + Send + Sync {
    async fn db_get_hotels(&self) -> Result<Vec<DbHotelDTO>, HotelInternalServiceError>;
    async fn db_get_hotel_room_types(
        &self,
    ) -> Result<Vec<DbHotelRoomTypeDTO>, HotelInternalServiceError>;
}
