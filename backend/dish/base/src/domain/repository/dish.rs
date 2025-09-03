use async_trait::async_trait;
use shared::Verified;
use shared::domain::model::dish::Dish;
use shared::domain::model::train::TrainNumber;
use shared::domain::{Repository, RepositoryError};

#[async_trait]
pub trait DishRepository: Repository<Dish> {
    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Vec<Dish>, RepositoryError>;
}
