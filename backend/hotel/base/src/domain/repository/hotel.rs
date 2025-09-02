use crate::domain::model::hotel::{Hotel, HotelId};
use async_trait::async_trait;
use shared::domain::model::city::CityId;
use shared::domain::model::station::StationId;
use shared::domain::{Repository, RepositoryError};
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
}
