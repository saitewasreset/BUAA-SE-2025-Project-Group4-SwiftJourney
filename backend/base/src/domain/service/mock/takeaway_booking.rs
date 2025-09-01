#![cfg(test)]

use crate::domain::model::order::TakeawayOrder;
use crate::domain::service::takeaway_booking::{
    TakeawayBookingService, TakeawayBookingServiceError,
};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

mock! {
    pub TakeawayBookingService {}

    #[async_trait]
    impl TakeawayBookingService for TakeawayBookingService {
        async fn booking_takeaway(&self, order_uuid: Uuid) -> Result<(), TakeawayBookingServiceError>;
    async fn cancel_takeaway(&self, order_uuid: Uuid) -> Result<(), TakeawayBookingServiceError>;

    // 返回要退款的订单
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<TakeawayOrder>, TakeawayBookingServiceError>;
    }
}

pub fn mock_takeaway_booking_service() -> MockTakeawayBookingService {
    MockTakeawayBookingService::new()
}
