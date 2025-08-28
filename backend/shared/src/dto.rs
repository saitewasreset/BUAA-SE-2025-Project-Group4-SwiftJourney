use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CityDTO {
    pub id: Option<u64>,
    pub name: String,
    pub province: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StationDTO {
    pub id: Option<u64>,
    pub name: String,
    #[serde(rename = "cityId")]
    pub city_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveResultDTO {
    pub id: u64,
}
