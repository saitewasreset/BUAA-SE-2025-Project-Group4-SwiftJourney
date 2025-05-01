use crate::application::ApplicationError;
use crate::application::commands::train_data::{
    LoadCityCommand, LoadStationCommand, LoadTrainNumberCommand, LoadTrainTypeCommand,
};
use async_trait::async_trait;

#[async_trait]
pub trait TrainDataService: 'static + Send + Sync {
    fn is_debug_mode(&self) -> bool;
    async fn load_city(&self, command: LoadCityCommand) -> Result<(), Box<dyn ApplicationError>>;

    async fn load_station(
        &self,
        command: LoadStationCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn load_train_type(
        &self,
        command: LoadTrainTypeCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn load_train_number(
        &self,
        command: LoadTrainNumberCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}
