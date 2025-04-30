use crate::domain::model::city::{City, ProvinceName};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::CityData;

#[async_trait]
pub trait CityRepository: Repository<City> + 'static + Send + Sync {
    async fn load(&self) -> Result<Vec<City>, RepositoryError>;

    async fn find_by_name(&self, city_name: &str) -> Result<Vec<City>, RepositoryError>;

    async fn find_by_province(
        &self,
        province_name: ProvinceName,
    ) -> Result<Vec<City>, RepositoryError>;

    async fn save_raw(&self, city_data: CityData) -> Result<(), RepositoryError>;
}
