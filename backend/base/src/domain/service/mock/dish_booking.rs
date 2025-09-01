#![cfg(test)]

use crate::domain::model::order::DishOrder;
use crate::domain::service::dish_booking::{DishBookingService, DishBookingServiceError};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

mock! {
    pub DishBookingService {}

    #[async_trait]
    impl DishBookingService for DishBookingService {
        async fn booking_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError>;
    async fn cancel_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError>;

    // 返回要退款的订单
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        atomic: bool,
    ) -> Result<Vec<DishOrder>, DishBookingServiceError>;
    }
}

pub fn mock_dish_booking_service() -> MockDishBookingService {
    MockDishBookingService::new()
}
