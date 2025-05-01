use crate::Verified;
use crate::domain::model::train::{SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::SeatId;
use crate::domain::repository::route::RouteRepository;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::{TrainNumberData, TrainTypeData};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[async_trait]
pub trait TrainRepository: Repository<Train> {
    async fn get_verified_train_number(&self) -> Result<HashSet<String>, RepositoryError>;
    async fn get_verified_train_type(&self) -> Result<HashSet<String>, RepositoryError>;

    async fn get_verified_seat_type(
        &self,
        train_id: TrainId,
    ) -> Result<HashSet<String>, RepositoryError>;

    async fn get_trains(&self) -> Result<Vec<Train>, RepositoryError>;

    async fn get_seat_id_map(
        &self,
        train_id: TrainId,
    ) -> Result<HashMap<SeatTypeName<Verified>, Vec<SeatId>>, RepositoryError>;

    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Train, RepositoryError>;

    async fn find_by_train_type(
        &self,
        train_type: TrainType<Verified>,
    ) -> Result<Vec<Train>, RepositoryError>;

    async fn save_raw_train_number<T: RouteRepository>(
        &self,
        train_number_data: TrainNumberData,
        route_repository: Arc<T>,
    ) -> Result<(), RepositoryError>;

    async fn save_raw_train_type(
        &self,
        train_type_data: TrainTypeData,
    ) -> Result<(), RepositoryError>;
}
