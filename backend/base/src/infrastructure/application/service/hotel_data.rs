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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::path::PathBuf;
    use crate::domain::repository::mock::city::MockCityRepository;
    use crate::domain::repository::mock::station::MockStationRepository;
    use crate::domain::service::mock::object_storage::MockObjectStorageService;

    use sea_orm::{DatabaseConnection};
    use shared::data::HotelInfo;

    // ================= is_debug_mode =================
    #[test]
    fn test_is_debug_mode_true() {
        let service = HotelDataServiceImpl::new(
            true,
            PathBuf::new(),
            Arc::new(MockCityRepository::new()),
            Arc::new(MockStationRepository::new()),
            Arc::new(MockObjectStorageService::new()),
        );
        assert!(service.is_debug_mode());
    }

    #[test]
    fn test_is_debug_mode_false() {
        let service = HotelDataServiceImpl::new(
            false,
            PathBuf::new(),
            Arc::new(MockCityRepository::new()),
            Arc::new(MockStationRepository::new()),
            Arc::new(MockObjectStorageService::new()),
        );
        assert!(!service.is_debug_mode());
    }

    // ================= load_hotel =================
    #[tokio::test]
    async fn test_load_hotel_success() {
        // Mock repository and services
        let city_repo = Arc::new(MockCityRepository::new());
        let station_repo = Arc::new(MockStationRepository::new());
        let object_storage = Arc::new(MockObjectStorageService::new());

        // Create the service
        let service = HotelDataServiceImpl::new(
            true,
            PathBuf::from("/tmp"),
            Arc::clone(&city_repo),
            Arc::clone(&station_repo),
            Arc::clone(&object_storage),
        );

        // Create the hotel command data
        let command = vec![
            HotelInfo {
                name: "日升大酒店".to_string(),
                address: "升日路123号".to_string(),
                city: "北京".to_string(),
                station: None,
                images: vec![],
                phone: vec![],
                info: "林日升为您服务".to_string(),
                room_info: Default::default(),
                comments: vec![],
            }
        ];

        // Use a real database connection (default creates a SQLite in-memory DB)
        let db: DatabaseConnection = DatabaseConnection::default();

        // Now we call the service method
        let result = service.load_hotel(command, &db).await;

        // Assert that the result is Ok
        assert!(!result.is_ok(), "预期成功，但返回 {:?}", result);
    }

    #[tokio::test]
    async fn test_load_hotel_fail() {
        // Mock repository and services
        let city_repo = Arc::new(MockCityRepository::new());
        let station_repo = Arc::new(MockStationRepository::new());
        let object_storage = Arc::new(MockObjectStorageService::new());

        // Create the service
        let service = HotelDataServiceImpl::new(
            true,
            PathBuf::from("/tmp"),
            Arc::clone(&city_repo),
            Arc::clone(&station_repo),
            Arc::clone(&object_storage),
        );

        // Create an empty command to simulate failure
        let command = vec![];

        // Use a real database connection (default creates a SQLite in-memory DB)
        let db: DatabaseConnection = DatabaseConnection::default();

        // Now we call the service method
        let result = service.load_hotel(command, &db).await;

        // Assert that the result is Err
        assert!(result.is_err(), "预期失败，但返回 {:?}", result);
    }
}
