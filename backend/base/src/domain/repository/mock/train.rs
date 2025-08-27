#![cfg(test)]

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

use crate::domain::model::train::{SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::{SeatId, SeatLocationInfo};
use crate::domain::repository::train::TrainRepository;
use crate::domain::{Repository, RepositoryError};
use crate::Verified;
use mockall::mock;

// 自动生成 Mock 类型
mock! {
    pub TrainRepository {}

    // 实现 Repository<Train>
    #[async_trait]
    impl Repository<Train> for TrainRepository {
        async fn find(&self, id: TrainId) -> Result<Option<Train>, RepositoryError>;
        async fn remove(&self, aggregate: Train) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut Train) -> Result<TrainId, RepositoryError>;
    }

    // 实现 TrainRepository
    #[async_trait]
    impl TrainRepository for TrainRepository {
        async fn get_verified_train_number(&self) -> Result<HashSet<String>, RepositoryError>;
        async fn get_verified_train_type(&self) -> Result<HashSet<String>, RepositoryError>;
        async fn get_verified_seat_type(&self, train_id: TrainId) -> Result<HashSet<String>, RepositoryError>;
        async fn get_trains(&self) -> Result<Vec<Train>, RepositoryError>;
        async fn get_seat_id_map(
            &self,
            train_id: TrainId,
        ) -> Result<HashMap<SeatTypeName<Verified>, Vec<(SeatId, SeatLocationInfo)>>, RepositoryError>;
        async fn find_by_train_number(
            &self,
            train_number: TrainNumber<Verified>,
        ) -> Result<Train, RepositoryError>;
        async fn find_by_train_type(
            &self,
            train_type: TrainType<Verified>,
        ) -> Result<Vec<Train>, RepositoryError>;
    }
}

// Helper 构造函数，方便测试时快速生成
pub fn mock_train_repo() -> MockTrainRepository {
    MockTrainRepository::new()
}
