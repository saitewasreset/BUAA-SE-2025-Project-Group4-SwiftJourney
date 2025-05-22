use crate::domain::model::route::{RouteId, Stop};
use crate::domain::model::takeaway::TakeawayShop;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::object_storage::ObjectStorageService;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::{DishData, TakeawayData};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait TakeawayShopRepository: Repository<TakeawayShop> {
    async fn find_by_train_route(
        &self,
        route_id: RouteId,
    ) -> Result<HashMap<Stop, Vec<TakeawayShop>>, RepositoryError>;

    async fn save_many_atomic(&self, entities: Vec<TakeawayShop>) -> Result<(), RepositoryError>;

    async fn save_raw_takeaway<S: StationRepository, OS: ObjectStorageService>(
        &self,
        data: TakeawayData,
        data_path: &Path,
        station_repository: Arc<S>,
        object_storage_service: Arc<OS>,
    ) -> Result<(), RepositoryError>;
}
