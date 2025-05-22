use crate::domain::model::route::{RouteId, Stop};
use crate::domain::model::takeaway::TakeawayShop;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait TakeawayShopRepository: Repository<TakeawayShop> {
    async fn find_by_train_route(
        &self,
        route_id: RouteId,
    ) -> Result<HashMap<Stop, Vec<TakeawayShop>>, RepositoryError>;
}
