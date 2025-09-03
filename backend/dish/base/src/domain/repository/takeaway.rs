use async_trait::async_trait;
use shared::domain::model::route::{RouteId, Stop};
use shared::domain::model::takeaway::TakeawayShop;
use shared::domain::{Repository, RepositoryError};
use std::collections::HashMap;

#[async_trait]
pub trait TakeawayShopRepository: Repository<TakeawayShop> {
    async fn find_by_train_route(
        &self,
        route_id: RouteId,
    ) -> Result<HashMap<Stop, Vec<TakeawayShop>>, RepositoryError>;

    async fn save_many_atomic(&self, entities: Vec<TakeawayShop>) -> Result<(), RepositoryError>;
}
