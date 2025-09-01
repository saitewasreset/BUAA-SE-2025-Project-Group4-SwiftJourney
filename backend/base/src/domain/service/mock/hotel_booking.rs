#![cfg(test)]

use crate::domain::model::hotel::{HotelDateRange, HotelId, HotelRoomStatus, HotelRoomTypeId};
use crate::domain::model::order::HotelOrder;
use crate::domain::service::hotel_booking::{HotelBookingService, HotelBookingServiceError};
use async_trait::async_trait;
use mockall::mock;
use std::collections::HashMap;
use uuid::Uuid;

mock! {
    pub HotelBookingService {}

    #[async_trait]
    impl HotelBookingService for HotelBookingService {
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
}

pub fn mock_hotel_booking_service() -> MockHotelBookingService {
    MockHotelBookingService::new()
}
