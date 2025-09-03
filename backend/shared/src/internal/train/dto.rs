use chrono::NaiveDate;
use sea_orm::prelude::{DateTimeWithTimeZone, Decimal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainDTO {
    pub id: u64,
    pub number: String,
    pub train_type: String,
    pub seats: HashMap<String, SeatTypeDTO>,
    pub default_route_id: u64,
    pub default_origin_departure_time: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SeatTypeDTO {
    pub seat_type_id: u64,
    pub type_name: String,
    pub capacity: u32,
    pub price: Decimal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainScheduleDTO {
    pub id: u64,
    pub train_id: u64,
    pub date: NaiveDate,
    pub origin_departure_time: i32,
    pub route_id: u64,
    pub seat_availability_map: HashMap<StationRangeDTO, HashMap<SeatTypeDTO, u64>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalArrivalTimeDTO {
    pub time: DateTimeWithTimeZone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StationRangeDTO {
    pub from_station_id: u64,
    pub to_station_id: u64,
}
