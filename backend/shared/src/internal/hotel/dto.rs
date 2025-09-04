use rust_decimal::Decimal;
use sea_orm::prelude::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbHotelDTO {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub city_id: i32,
    pub station_id: i32,
    pub address: String,
    pub phone: Json,
    pub images: Json,
    pub total_rating_count: i32,
    pub total_booking_count: i32,
    pub info: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbHotelRoomTypeDTO {
    pub id: i32,
    pub type_name: String,
    pub capacity: i32,
    pub price: Decimal,
    pub hotel_id: i32,
}
