use crate::domain::model::city::CityId;
use crate::domain::model::station::Station;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

#[async_trait]
pub trait StationRepository: Repository<Station> {
    async fn load(&self) -> Result<Vec<Station>, RepositoryError>;

    async fn find_by_city(&self, city_id: CityId) -> Result<Vec<Station>, RepositoryError>;

    async fn find_by_name(&self, station_name: &str) -> Result<Option<Station>, RepositoryError>;
}
