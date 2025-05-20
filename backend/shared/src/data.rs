use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// city -> province
pub type CityData = HashMap<String, String>;
pub type StationData = Vec<StationDataItem>;
pub type TrainTypeData = Vec<TrainTypeInfoItem>;
pub type TrainNumberData = Vec<TrainNumberInfoItem>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StationDataItem {
    pub name: String,
    pub city: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeatLocationInfo {
    pub carriage: i32,
    pub row: i32,
    pub location: char,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeatInfo {
    pub description: SeatLocationInfo,
    pub price: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TrainTypeInfoItem {
    pub id: String,
    pub name: String,
    // 等级 -> 位置
    pub seat: HashMap<String, HashMap<char, Vec<SeatInfo>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RouteStationInfo {
    pub order: u32,
    pub station: String,

    #[serde(rename = "arrivalTime")]
    pub arrival_time: u32,
    #[serde(rename = "depatureTime")]
    pub departure_time: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrainNumberInfoItem {
    pub train_number: String,
    pub train_type: String,
    #[serde(rename = "originDepatureTime")]
    pub origin_departure_time: u32,
    pub route: Vec<RouteStationInfo>,
}
