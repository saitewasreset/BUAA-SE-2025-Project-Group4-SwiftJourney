#![cfg(test)]

use crate::domain::model::order::TrainOrder;
use crate::domain::service::train_booking::{TrainBookingService, TrainBookingServiceError};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

mock! {
    pub TrainBookingService {}

    #[async_trait]
    impl TrainBookingService for TrainBookingService {
        async fn booking_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError>;
    async fn cancel_ticket(&self, order_uuid: Uuid) -> Result<(), TrainBookingServiceError>;

    // 返回要退款的订单
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<TrainOrder>, TrainBookingServiceError>;
    }
}

pub fn mock_train_booking_service() -> MockTrainBookingService {
    MockTrainBookingService::new()
}
