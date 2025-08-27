#![cfg(test)]

use async_trait::async_trait;
use mockall::mock;

use crate::domain::model::dish::{Dish, DishId};
use crate::domain::model::train::TrainNumber;
use crate::domain::repository::dish::DishRepository;
use crate::domain::{Repository, RepositoryError};
use crate::Verified;

mock! {
    pub DishRepository {}

    #[async_trait]
    impl Repository<Dish> for DishRepository {
        async fn find(&self, id: DishId) -> Result<Option<Dish>, RepositoryError>;
        async fn remove(&self, aggregate: Dish) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut Dish) -> Result<DishId, RepositoryError>;
    }

    #[async_trait]
    impl DishRepository for DishRepository {
        async fn find_by_train_number(
            &self,
            train_number: TrainNumber<Verified>,
        ) -> Result<Vec<Dish>, RepositoryError>;
    }
}

pub fn mock_dish_repo() -> MockDishRepository {
    MockDishRepository::new()
}
