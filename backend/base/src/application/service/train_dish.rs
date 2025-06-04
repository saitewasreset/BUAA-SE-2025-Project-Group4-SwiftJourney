use crate::application::ApplicationError;
use crate::application::commands::train_dish::OrderTrainDishCommand;
use crate::application::service::transaction::TransactionInfoDTO;
use crate::domain::model::dish::{DishId, DishTime};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::station::StationId;
use crate::domain::model::takeaway::TakeawayDishId;
use crate::domain::model::train::TrainId;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TrainDishApplicationServiceError {
    #[error("Invalid dish name: {0}")]
    InvalidDishName(String),
    #[error("Invalid dish name")]
    InvalidAmount,
    #[error("Invalid takeaway station: {0}")]
    InvalidTakeawayStation(String),
    #[error("Invalid takeaway shop name: {0}")]
    InvalidTakeawayShopName(String),
    #[error("Invalid takeaway name: {0}")]
    InvalidTakeawayName(String),
    #[error("No related train order found")]
    NoRelatedTrainOrder,
}

impl ApplicationError for TrainDishApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainDishApplicationServiceError::InvalidDishName(_) => 22001,
            TrainDishApplicationServiceError::InvalidAmount => 22002,
            TrainDishApplicationServiceError::InvalidTakeawayStation(_) => 22003,
            TrainDishApplicationServiceError::InvalidTakeawayShopName(_) => 22004,
            TrainDishApplicationServiceError::InvalidTakeawayName(_) => 22005,
            TrainDishApplicationServiceError::NoRelatedTrainOrder => 22006,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DishOrderRequestDTO {
    pub name: String,
    pub personal_id: Uuid,
    pub amount: u32,
    pub dish_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerifiedDishOrderRequest {
    pub dish_id: DishId,
    pub train_id: TrainId,
    pub personal_id: PersonalInfoId,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub dish_time: DishTime,
    pub active_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TakeawayOrderRequestDTO {
    pub station: String,
    pub arrival_time: String,
    pub shop_name: String,
    pub name: String,
    pub personal_id: Uuid,
    pub amount: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerifiedTakeawayOrderRequest {
    pub takeaway_dish_id: TakeawayDishId,
    pub train_id: TrainId,
    pub station_id: StationId,
    pub personal_id: PersonalInfoId,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub active_time: DateTimeWithTimeZone,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrainDishOrderRequestDTO {
    pub train_number: String,
    pub origin_departure_time: String,

    pub dishes: Vec<DishOrderRequestDTO>,
    pub takeaway: Vec<TakeawayOrderRequestDTO>,
}

#[async_trait]
pub trait TrainDishApplicationService: 'static + Send + Sync {
    async fn order_dish(
        &self,
        command: OrderTrainDishCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>>;
}
