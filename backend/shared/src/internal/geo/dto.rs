use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbCityDTO {
    pub id: i32,
    pub name: String,
    pub province: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbStationDTO {
    pub id: i32,
    pub name: String,
    pub city_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CityDTO {
    pub city_id: u64,
    pub name: String,
    pub province: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StationDTO {
    pub station_id: u64,
    pub name: String,
    pub city_id: u64,
}
