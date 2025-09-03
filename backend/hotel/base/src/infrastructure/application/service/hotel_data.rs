use crate::application::commands::hotel_data::LoadHotelCommand;
use crate::application::service::hotel_data::HotelDataService;
use crate::infrastructure::repository::hotel::save_raw_hotel;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::application_error::{ApplicationError, GeneralError};
use shared::ports::geo::GeoPort;
use shared::ports::object_storage::ObjectStoragePort;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct HotelDataServiceImpl<GP, OP>
where
    GP: GeoPort,
    OP: ObjectStoragePort,
{
    debug: bool,
    data_base_path: PathBuf,
    geo_port: Arc<GP>,
    object_storage_port: Arc<OP>,
}

impl<GP, OP> HotelDataServiceImpl<GP, OP>
where
    GP: GeoPort,
    OP: ObjectStoragePort,
{
    pub fn new(
        debug: bool,
        data_base_path: PathBuf,
        geo_port: Arc<GP>,
        object_storage_port: Arc<OP>,
    ) -> Self {
        HotelDataServiceImpl {
            debug,
            data_base_path,
            geo_port,
            object_storage_port,
        }
    }
}

#[async_trait]
impl<GP, OP> HotelDataService for HotelDataServiceImpl<GP, OP>
where
    GP: GeoPort,
    OP: ObjectStoragePort,
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
            Arc::clone(&self.object_storage_port),
            Arc::clone(&self.geo_port),
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
