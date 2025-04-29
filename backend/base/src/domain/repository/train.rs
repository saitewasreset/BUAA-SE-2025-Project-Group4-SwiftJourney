use crate::Verified;
use crate::domain::model::train::{SeatType, Train, TrainId, TrainNumber, TrainType};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use std::collections::HashSet;

#[async_trait]
pub trait TrainRepository: Repository<Train> {
    async fn get_verified_train_number(
        &self,
    ) -> Result<HashSet<TrainNumber<Verified>>, RepositoryError>;
    async fn get_verified_train_type(
        &self,
    ) -> Result<HashSet<TrainType<Verified>>, RepositoryError>;

    async fn get_verified_seat_type(
        &self,
        train_id: TrainId,
    ) -> Result<HashSet<SeatType>, RepositoryError>;

    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Option<Train>, RepositoryError>;

    async fn find_by_train_type(
        &self,
        train_type: TrainType<Verified>,
    ) -> Result<Vec<Train>, RepositoryError>;
}
