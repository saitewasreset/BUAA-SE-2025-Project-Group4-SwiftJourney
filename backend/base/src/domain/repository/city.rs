use crate::domain::model::city::{City, CityName, ProvinceName};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

#[async_trait]
pub trait CityRepository: Repository<City> {
    async fn load(&self) -> Result<Vec<City>, RepositoryError>;

    async fn find_by_name(&self, city_name: CityName) -> Result<Vec<City>, RepositoryError>;

    async fn find_by_province(
        &self,
        province_name: ProvinceName,
    ) -> Result<Vec<City>, RepositoryError>;
}
