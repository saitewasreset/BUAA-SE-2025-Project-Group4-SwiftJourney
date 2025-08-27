#![cfg(test)]

use async_trait::async_trait;

use crate::domain::model::order::Order;
use crate::domain::model::user::UserId;
use crate::domain::service::order::order_dto::OrderInfoDto;
use crate::domain::service::order::OrderService;
use crate::domain::RepositoryError;

use mockall::mock;

// 使用 mockall 创建 OrderService 的 Mock
mock! {
    pub OrderService {}

    #[async_trait]
    impl OrderService for OrderService {
        async fn convert_order_to_dto(
            &self,
            order: Box<dyn Order>,
        ) -> Result<OrderInfoDto, RepositoryError>;

        async fn verify_train_order(
            &self,
            user_id: UserId,
            train_number: String,
            origin_departure_time: sea_orm::prelude::DateTimeWithTimeZone,
        ) -> Result<bool, RepositoryError>;
    }
}

// Helper 函数，方便在测试中生成 Arc<MockOrderService>
pub fn mock_order_service() -> MockOrderService {
    MockOrderService::new()
}


