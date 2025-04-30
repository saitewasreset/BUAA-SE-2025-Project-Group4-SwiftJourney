use crate::application::ApplicationError;
use crate::application::commands::train_data::{
    LoadCityCommand, LoadStationCommand, LoadTrainNumberCommand, LoadTrainTypeCommand,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct LoadTimeInfo {
    // in seconds
    pub parse: f64,
    pub load: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct LoadResultDTO {
    pub loaded: usize,
    pub total: usize,
    #[serde(rename = "timeInfo")]
    pub time_info: LoadTimeInfo,
}

#[async_trait]
pub trait TrainDataService: 'static + Send + Sync {
    fn is_debug_mode(&self) -> bool;
    async fn load_city(
        &self,
        command: LoadCityCommand,
    ) -> Result<LoadResultDTO, Box<dyn ApplicationError>>;

    async fn load_station(
        &self,
        command: LoadStationCommand,
    ) -> Result<LoadResultDTO, Box<dyn ApplicationError>>;

    async fn load_train_type(
        &self,
        command: LoadTrainTypeCommand,
    ) -> Result<LoadResultDTO, Box<dyn ApplicationError>>;

    async fn load_train_number(
        &self,
        command: LoadTrainNumberCommand,
    ) -> Result<LoadResultDTO, Box<dyn ApplicationError>>;
}
