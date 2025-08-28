use crate::application::commands::hotel_data::LoadHotelCommand;
use crate::application::service::hotel_data::HotelDataService;
use crate::application::{ApplicationError, GeneralError};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::service::object_storage::ObjectStorageService;
use crate::infrastructure::repository::hotel::save_raw_hotel;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct HotelDataServiceImpl<C, S, OS>
where
    C: CityRepository,
    S: StationRepository,
    OS: ObjectStorageService,
{
    debug: bool,
    data_base_path: PathBuf,
    city_repository: Arc<C>,
    station_repository: Arc<S>,
    object_storage_service: Arc<OS>,
}

impl<C, S, OS> HotelDataServiceImpl<C, S, OS>
where
    C: CityRepository,
    S: StationRepository,
    OS: ObjectStorageService,
{
    pub fn new(
        debug: bool,
        data_base_path: PathBuf,
        city_repository: Arc<C>,
        station_repository: Arc<S>,
        object_storage_service: Arc<OS>,
    ) -> Self {
        HotelDataServiceImpl {
            debug,
            data_base_path,
            city_repository,
            station_repository,
            object_storage_service,
        }
    }
}

#[async_trait]
impl<C, S, OS> HotelDataService for HotelDataServiceImpl<C, S, OS>
where
    C: CityRepository,
    S: StationRepository,
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
