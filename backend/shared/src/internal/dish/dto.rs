use rust_decimal::Decimal;
use sea_orm::prelude::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbDishDTO {
    pub id: i32,
    pub train_id: i32,
    pub r#type: String,
    pub time: String,
    pub name: String,
    pub price: Decimal,
    pub images: Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbTakeawayDishDTO {
    pub id: i32,
    pub name: String,
    pub dish_type: String,
    pub price: Decimal,
    pub takeaway_shop_id: i32,
    pub images: Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbTakeawayShopDTO {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub station_id: i32,
    pub images: Json,
}
