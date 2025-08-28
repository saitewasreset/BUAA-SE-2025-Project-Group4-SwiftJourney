#![cfg(test)]

use mockall::mock;

use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::StationData;
use crate::domain::repository::station::StationRepository;

mock! {
    pub StationRepository {}
    
    #[async_trait]
    impl StationRepository for StationRepository {
        
        async fn load(&self) -> Result<Vec<Station>, RepositoryError>;
        
        async fn find_by_city(&self, city_id: CityId) -> Result<Vec<Station>, RepositoryError>;
        
        async fn find_by_name(&self, station_name: &str) -> Result<Option<Station>, RepositoryError>;
        
        async fn save_raw(&self, station_data: StationData) -> Result<(), RepositoryError>;
    }
    
    #[async_trait]
    impl Repository<Station> for StationRepository {
        async fn find(&self, id: StationId) -> Result<Option<Station>, RepositoryError>;
        async fn remove(&self, aggregate: Station) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut Station) -> Result<StationId, RepositoryError>;
    }
}

pub fn mock_station_repository() -> MockStationRepository {
    MockStationRepository::new()
}