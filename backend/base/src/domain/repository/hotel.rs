use crate::domain::model::city::CityId;
use crate::domain::model::hotel::{Hotel, HotelId};
use crate::domain::model::station::StationId;
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::service::object_storage::ObjectStorageService;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::HotelData;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait HotelRepository: Repository<Hotel> {
    async fn get_id_by_uuid(&self, uuid: Uuid) -> Result<Option<HotelId>, RepositoryError>;
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Hotel>, RepositoryError>;
    async fn find_by_city(
        &self,
        city_id: CityId,
        name_pattern: Option<&str>,
    ) -> Result<Vec<Hotel>, RepositoryError>;
    async fn find_by_station(
        &self,
        station_id: StationId,
        name_pattern: Option<&str>,
    ) -> Result<Vec<Hotel>, RepositoryError>;

    async fn save_raw_hotel<C: CityRepository, S: StationRepository, OS: ObjectStorageService>(
        &self,
        city_repository: Arc<C>,
        station_repository: Arc<S>,
        object_storage: Arc<OS>,
        data_base_path: &Path,
        hotel_data: HotelData,
    ) -> Result<(), RepositoryError>;
}
