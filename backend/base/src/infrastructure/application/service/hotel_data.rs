use crate::application::commands::hotel_data::LoadHotelCommand;
use crate::application::service::hotel_data::HotelDataService;
use crate::application::{ApplicationError, GeneralError};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::service::object_storage::ObjectStorageService;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct HotelDataServiceImpl<C, S, OS, H>
where
    C: CityRepository,
    S: StationRepository,
    OS: ObjectStorageService,
    H: HotelRepository,
{
    debug: bool,
    data_base_path: PathBuf,
    city_repository: Arc<C>,
    station_repository: Arc<S>,
    object_storage_service: Arc<OS>,
    hotel_repository: Arc<H>,
}

impl<C, S, OS, H> HotelDataServiceImpl<C, S, OS, H>
where
    C: CityRepository,
    S: StationRepository,
    OS: ObjectStorageService,
    H: HotelRepository,
{
    pub fn new(
        debug: bool,
        data_base_path: PathBuf,
        city_repository: Arc<C>,
        station_repository: Arc<S>,
        object_storage_service: Arc<OS>,
        hotel_repository: Arc<H>,
    ) -> Self {
        HotelDataServiceImpl {
            debug,
            data_base_path,
            city_repository,
            station_repository,
            object_storage_service,
            hotel_repository,
        }
    }
}

#[async_trait]
impl<C, S, OS, H> HotelDataService for HotelDataServiceImpl<C, S, OS, H>
where
    C: CityRepository,
    S: StationRepository,
    OS: ObjectStorageService,
    H: HotelRepository,
{
    fn is_debug_mode(&self) -> bool {
        self.debug
    }

    #[instrument(skip_all)]
    async fn load_hotel(&self, command: LoadHotelCommand) -> Result<(), Box<dyn ApplicationError>> {
        self.hotel_repository
            .save_raw_hotel(
                Arc::clone(&self.city_repository),
                Arc::clone(&self.station_repository),
                Arc::clone(&self.object_storage_service),
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
