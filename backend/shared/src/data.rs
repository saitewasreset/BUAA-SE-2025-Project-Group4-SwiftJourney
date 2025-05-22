use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// city -> province
pub type CityData = HashMap<String, String>;
pub type StationData = Vec<StationDataItem>;
pub type TrainTypeData = Vec<TrainTypeInfoItem>;
pub type TrainNumberData = Vec<TrainNumberInfoItem>;

pub type HotelData = Vec<HotelInfo>;

// train number -> [DishInfo]
pub type DishData = HashMap<String, Vec<DishInfo>>;

// station -> shop -> [TakeawayDishInfo]
pub type TakeawayData = HashMap<String, HashMap<String, Vec<TakeawayDishInfo>>>;

pub type RawDishTakeawayData = Vec<RawDishTakeawayInfo>;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HotelRoomType {
    pub capacity: i32,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HotelComment {
    pub time: String,
    pub rating: f64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HotelInfo {
    pub name: String,
    pub address: String,
    pub city: String,
    pub station: Option<String>,

    pub images: Vec<String>,

    pub phone: Vec<String>,

    pub info: String,

    pub room_info: HashMap<String, HotelRoomType>,

    pub comments: Vec<HotelComment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DishInfo {
    pub available_time: Vec<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub dish_type: String,
    pub picture: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TakeawayDishInfo {
    pub name: String,
    pub picture: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RawDishTakeawayInfo {
    pub train_number: String,
    pub dish_info: Vec<DishInfo>,
    pub takeaway_info: HashMap<String, HashMap<String, Vec<TakeawayDishInfo>>>,
}
