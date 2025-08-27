#![cfg(test)]

use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::model::route::{RouteId, Stop};
use crate::domain::model::takeaway::TakeawayShop;
use crate::domain::model::takeaway::TakeawayShopId;
use crate::domain::repository::takeaway::TakeawayShopRepository;
use crate::domain::{Repository, RepositoryError};
use mockall::mock;

// 自动生成 Mock 类型
mock! {
    pub TakeawayShopRepository {}

    #[async_trait]
    impl Repository<TakeawayShop> for TakeawayShopRepository {
        async fn find(&self, id: TakeawayShopId) -> Result<Option<TakeawayShop>, RepositoryError>;
        async fn remove(&self, aggregate: TakeawayShop) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut TakeawayShop) -> Result<TakeawayShopId, RepositoryError>;
    }

    #[async_trait]
    impl TakeawayShopRepository for TakeawayShopRepository {
        async fn find_by_train_route(
            &self,
            route_id: RouteId,
        ) -> Result<HashMap<Stop, Vec<TakeawayShop>>, RepositoryError>;

        async fn save_many_atomic(
            &self,
            entities: Vec<TakeawayShop>
        ) -> Result<(), RepositoryError>;
    }
}

// Helper 构造函数，方便在测试中生成 Arc<MockTakeawayShopRepository>
pub fn mock_takeaway_shop_repo() -> MockTakeawayShopRepository {
    MockTakeawayShopRepository::new()
}
