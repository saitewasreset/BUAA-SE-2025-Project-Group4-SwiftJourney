use crate::domain::RepositoryError;
use crate::domain::model::route::RouteId;
use crate::domain::model::train::{SeatType, SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::SeatId;
use crate::domain::service::ServiceError;
use crate::{Unverified, Verified};
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainTypeConfigurationServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid seat type {1} for train id: {0}")]
    InvalidSeatType(u64, String),
    #[error("invalid train number: {0}")]
    InvalidTrainNumber(String),
    #[error("invalid train type: {0}")]
    InvalidTrainType(String),
    #[error("train type {0} already exists")]
    TrainTypeExists(u64),
    #[error("no such train id: {0}")]
    NoSuchTrainId(u64),
}

impl From<RepositoryError> for TrainTypeConfigurationServiceError {
    fn from(value: RepositoryError) -> Self {
        Self::InfrastructureError(ServiceError::RepositoryError(value))
    }
}

#[async_trait]
pub trait TrainTypeConfigurationService {
    async fn verify_seat_type_name(
        &self,
        train_id: TrainId,
        seat_type_name: SeatTypeName<Unverified>,
    ) -> Result<SeatTypeName<Verified>, TrainTypeConfigurationServiceError>;

    async fn verify_train_number(
        &self,
        train_number: TrainNumber<Unverified>,
    ) -> Result<TrainNumber<Verified>, TrainTypeConfigurationServiceError>;

    async fn verify_train_type(
        &self,
        train_type: TrainType<Unverified>,
    ) -> Result<TrainType<Verified>, TrainTypeConfigurationServiceError>;

    async fn get_seat_id_map(
        &self,
        train_id: TrainId,
    ) -> Result<HashMap<SeatTypeName<Verified>, Vec<SeatId>>, TrainTypeConfigurationServiceError>;

    async fn get_trains(&self) -> Result<Vec<Train>, TrainTypeConfigurationServiceError>;

    async fn get_train_by_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Train, TrainTypeConfigurationServiceError>;

    async fn add_train_type(
        &self,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
    ) -> Result<TrainId, TrainTypeConfigurationServiceError>;

    async fn modify_train_type(
        &self,
        train_id: TrainId,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
    ) -> Result<(), TrainTypeConfigurationServiceError>;

    async fn remove_train_type(
        &self,
        train: Train,
    ) -> Result<(), TrainTypeConfigurationServiceError>;
}
