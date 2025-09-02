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
