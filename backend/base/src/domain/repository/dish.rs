use crate::Verified;
use crate::domain::model::dish::Dish;
use crate::domain::model::train::TrainNumber;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

#[async_trait]
pub trait DishRepository: Repository<Dish> {
    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Vec<Dish>, RepositoryError>;
}
