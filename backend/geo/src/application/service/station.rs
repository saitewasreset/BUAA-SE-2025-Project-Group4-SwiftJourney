use crate::application::ApplicationError;
use crate::application::commands::station::LoadStationCommand;
use async_trait::async_trait;

#[async_trait]
pub trait StationService: 'static + Send + Sync {
    async fn load_station(
        &self,
        command: LoadStationCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}
