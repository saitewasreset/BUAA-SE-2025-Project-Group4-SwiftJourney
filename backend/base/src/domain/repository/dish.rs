use crate::Verified;
use crate::domain::model::dish::Dish;
use crate::domain::model::train::TrainNumber;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::object_storage::ObjectStorageService;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::DishData;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait DishRepository: Repository<Dish> {
    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Vec<Dish>, RepositoryError>;

    async fn save_raw_dish<T: TrainRepository, OS: ObjectStorageService>(
        &self,
        data: DishData,
        data_path: &Path,
        train_repository: Arc<T>,
        object_storage_service: Arc<OS>,
    ) -> Result<(), RepositoryError>;
}
