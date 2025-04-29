use crate::Verified;
use crate::domain::model::train::{Train, TrainNumber, TrainType};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

#[async_trait]
pub trait TrainRepository: Repository<Train> {
    async fn get_verified_train_number(
        &self,
    ) -> Result<Vec<TrainNumber<Verified>>, RepositoryError>;
    async fn get_verified_train_type(&self) -> Result<Vec<TrainType<Verified>>, RepositoryError>;

    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Option<Train>, RepositoryError>;

    async fn find_by_train_type(
        &self,
        train_type: TrainType<Verified>,
    ) -> Result<Vec<Train>, RepositoryError>;
}
