use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::model::{
    dish::{DishId, DishTime},
    personal_info::PersonalInfoId,
    station::StationId,
    takeaway::TakeawayDishId,
    train::TrainId,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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
pub struct TakeawayOrderRequestDTO {
    pub station: String,
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
pub struct TrainDishOrderRequestDTO {
    pub train_number: String,
    pub origin_departure_time: String,

    pub dishes: Vec<DishOrderRequestDTO>,
    pub takeaway: Vec<TakeawayOrderRequestDTO>,
}
