use crate::domain::RepositoryError;
use crate::domain::model::hotel::{
    HotelDateRange, HotelId, HotelRoomStatus, HotelRoomTypeId, HotelRoomTypeStr,
};
use crate::domain::model::order::{HotelOrder, OrderStatus};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HotelBookingServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("no available room for order uuid: {0}")]
    NoAvailableRoom(Uuid),
    #[error("no order found for order uuid: {0}")]
    InvalidOrder(Uuid),
    #[error("invalid order status for order uuid: {0}, status: {1}")]
    InvalidOrderStatus(Uuid, OrderStatus),
    #[error("no hotel found for id: {0}")]
    InvalidHotelId(HotelId),
}

impl From<RepositoryError> for HotelBookingServiceError {
    fn from(value: RepositoryError) -> Self {
        HotelBookingServiceError::InfrastructureError(ServiceError::RepositoryError(value))
    }
}

#[async_trait]
pub trait HotelBookingService: 'static + Send + Sync {
    async fn get_available_room(
        &self,
        hotel_id: HotelId,
        booking_date_range: HotelDateRange,
    ) -> Result<HashMap<HotelRoomTypeId, HotelRoomStatus>, HotelBookingServiceError>;

    /// 预定酒店
    /// 订单状态应当由调用者修改
    async fn booking_hotel(&self, order_uuid: Uuid) -> Result<(), HotelBookingServiceError>;

    /// 取消酒店预定
    /// 订单状态应当由调用者修改
    async fn cancel_hotel(&self, order_uuid: Uuid) -> Result<(), HotelBookingServiceError>;

    /// 预定酒店组
    /// 订单状态应当由调用者修改
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<HotelOrder>, HotelBookingServiceError>;
}
