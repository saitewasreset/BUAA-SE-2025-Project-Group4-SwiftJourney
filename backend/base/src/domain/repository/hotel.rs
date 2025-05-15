use crate::domain::model::city::CityId;
use crate::domain::model::hotel::Hotel;
use crate::domain::model::station::StationId;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait HotelRepository: Repository<Hotel> {
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
