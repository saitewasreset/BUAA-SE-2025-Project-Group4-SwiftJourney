use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveCityProvinceMapCommand {
    pub city_province_map: std::collections::HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveStationCityMapCommand {
    pub station_city_map: std::collections::HashMap<String, String>,
}
