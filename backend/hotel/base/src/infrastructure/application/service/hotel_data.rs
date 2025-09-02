use crate::application::commands::hotel_data::LoadHotelCommand;
use crate::application::service::hotel_data::HotelDataService;
use crate::application::service::ports::geo::GeoPort;
use crate::infrastructure::repository::hotel::save_raw_hotel;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::application_error::{ApplicationError, GeneralError};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct HotelDataServiceImpl<GP, OS>
where
    GP: GeoPort,
    OS: ObjectStorageService,
{
    debug: bool,
    data_base_path: PathBuf,
    geo_port: Arc<GP>,
    object_storage_service: Arc<OS>,
}

impl<GP, OS> HotelDataServiceImpl<GP, OS>
where
    GP: GeoPort,
    OS: ObjectStorageService,
{
    pub fn new(
        debug: bool,
        data_base_path: PathBuf,
        geo_port: Arc<GP>,
        object_storage_service: Arc<OS>,
    ) -> Self {
        HotelDataServiceImpl {
            debug,
            data_base_path,
            geo_port,
            object_storage_service,
        }
    }
}

#[async_trait]
impl<GP, OS> HotelDataService for HotelDataServiceImpl<GP, OS>
where
    GP: GeoPort,
    OS: ObjectStorageService,
{
    fn is_debug_mode(&self) -> bool {
        self.debug
    }

    #[instrument(skip_all)]
    async fn load_hotel(
        &self,
        command: LoadHotelCommand,
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>> {
        save_raw_hotel(
            Arc::clone(&self.city_repository),
            Arc::clone(&self.station_repository),
            Arc::clone(&self.object_storage_service),
            db,
            &self.data_base_path,
            command,
        )
        .await
        .map_err(|e| {
            error!("Failed to save raw hotel data: {}", e);

            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })
    }
}
