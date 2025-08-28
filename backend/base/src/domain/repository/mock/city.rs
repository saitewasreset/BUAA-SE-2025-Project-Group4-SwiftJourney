#![cfg(test)]

use mockall::mock;

use crate::domain::model::city::{City, CityId, ProvinceName};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::CityData;
use crate::domain::repository::city::CityRepository;


mock! {
    pub CityRepository {}

    #[async_trait]
    impl Repository<City> for CityRepository {
        async fn find(&self, id: CityId) -> Result<Option<City>, RepositoryError>;
        async fn remove(&self, aggregate: City) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut City) -> Result<CityId, RepositoryError>;
    }
    
    #[async_trait]
    impl CityRepository for CityRepository {
        async fn load(&self) -> Result<Vec<City>, RepositoryError>;
        
        async fn find_by_name(&self, city_name: &str) -> Result<Vec<City>, RepositoryError>;
        
        async fn find_by_province(
            &self,
            province_name: ProvinceName,
        ) -> Result<Vec<City>, RepositoryError>;
        
        async fn save_raw(&self, city_data: CityData) -> Result<(), RepositoryError>;
    }
}

pub fn mock_city_repository() -> MockCityRepository {
    MockCityRepository::new()
}