use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::repository::city::CityRepository;
use crate::domain::{Identifiable, Repository, RepositoryError};
use async_trait::async_trait;
use serde::Deserialize;
use shared::dto::{CityDTO, SaveResultDTO};

#[derive(Clone)]
pub struct CityRepositoryHttpImpl {
    base_url: String,
    client: reqwest::Client,
}

impl CityRepositoryHttpImpl {
    pub fn new(base_url: impl Into<String>) -> Self {
        CityRepositoryHttpImpl {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/geo{}", self.base_url.trim_end_matches('/'), path)
    }
}

#[async_trait]
impl Repository<City> for CityRepositoryHttpImpl {
    async fn find(&self, id: CityId) -> Result<Option<City>, RepositoryError> {
        let url = self.url(&format!("/cities/{}", u64::from(id)));
        let resp = self.client.get(url).send().await.map_err(|e| RepositoryError::Db(e.into()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let dto: CityDTO = resp.json().await.map_err(|e| RepositoryError::Db(e.into()))?;
        Ok(Some(City::new(
            dto.id.map(Into::into),
            CityName::from(dto.name),
            ProvinceName::from(dto.province),
        )))
    }

    async fn remove(&self, aggregate: City) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let url = self.url(&format!("/cities/{}", u64::from(id)));
            self.client
                .delete(url)
                .send()
                .await
                .map_err(|e| RepositoryError::Db(e.into()))?
                .error_for_status()
                .map_err(|e| RepositoryError::Db(e.into()))?;
        }
        Ok(())
    }

    async fn save(&self, aggregate: &mut City) -> Result<CityId, RepositoryError> {
        let dto = CityDTO {
            id: aggregate.get_id().map(Into::into),
            name: aggregate.name().to_string(),
            province: aggregate.province().to_string(),
        };
        let url = self.url("/cities");
        let resp = self
            .client
            .post(url)
            .json(&dto)
            .send()
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let SaveResultDTO { id } = resp.json().await.map_err(|e| RepositoryError::Db(e.into()))?;
        aggregate.set_id(CityId::from(id));
        Ok(CityId::from(id))
    }
}

#[async_trait]
impl CityRepository for CityRepositoryHttpImpl {
    async fn load(&self) -> Result<Vec<City>, RepositoryError> {
        let url = self.url("/cities");
        let list: Vec<CityDTO> = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?
            .json()
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        Ok(list
            .into_iter()
            .map(|dto| City::new(dto.id.map(Into::into), CityName::from(dto.name), ProvinceName::from(dto.province)))
            .collect())
    }

    async fn find_by_name(&self, city_name: &str) -> Result<Vec<City>, RepositoryError> {
        #[derive(Deserialize)]
        struct Resp(Vec<CityDTO>);
        let url = self.url(&format!("/cities?name={}", urlencoding::encode(city_name)));
        let list: Vec<CityDTO> = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?
            .json()
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;
        Ok(list
            .into_iter()
            .map(|dto| City::new(dto.id.map(Into::into), CityName::from(dto.name), ProvinceName::from(dto.province)))
            .collect())
    }

    async fn find_by_province(
        &self,
        province_name: ProvinceName,
    ) -> Result<Vec<City>, RepositoryError> {
        let url = self.url(&format!("/cities?province={}", urlencoding::encode(&province_name)));
        let list: Vec<CityDTO> = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?
            .json()
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;
        Ok(list
            .into_iter()
            .map(|dto| City::new(dto.id.map(Into::into), CityName::from(dto.name), ProvinceName::from(dto.province)))
            .collect())
    }

    async fn save_raw(&self, _city_data: shared::data::CityData) -> Result<(), RepositoryError> {
        // Optional: could POST to /geo/load/city in debug mode if needed.
        Ok(())
    }
}
