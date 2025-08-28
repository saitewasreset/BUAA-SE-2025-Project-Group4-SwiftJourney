use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::repository::station::StationRepository;
use crate::domain::{Identifiable, Repository, RepositoryError};
use async_trait::async_trait;
use shared::dto::{SaveResultDTO, StationDTO};

#[derive(Clone)]
pub struct StationRepositoryHttpImpl {
    base_url: String,
    client: reqwest::Client,
}

impl StationRepositoryHttpImpl {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into(), client: reqwest::Client::new() }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/geo{}", self.base_url.trim_end_matches('/'), path)
    }
}

#[async_trait]
impl Repository<Station> for StationRepositoryHttpImpl {
    async fn find(&self, id: StationId) -> Result<Option<Station>, RepositoryError> {
        let url = self.url(&format!("/stations/{}", u64::from(id)));
        let resp = self.client.get(url).send().await.map_err(|e| RepositoryError::Db(e.into()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND { return Ok(None); }
        let dto: StationDTO = resp.json().await.map_err(|e| RepositoryError::Db(e.into()))?;
        Ok(Some(Station::new(dto.id.map(Into::into), dto.name, CityId::from(dto.city_id))))
    }

    async fn remove(&self, aggregate: Station) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let url = self.url(&format!("/stations/{}", u64::from(id)));
            self.client.delete(url).send().await.map_err(|e| RepositoryError::Db(e.into()))?
                .error_for_status().map_err(|e| RepositoryError::Db(e.into()))?;
        }
        Ok(())
    }

    async fn save(&self, aggregate: &mut Station) -> Result<StationId, RepositoryError> {
        let dto = StationDTO { id: aggregate.get_id().map(Into::into), name: aggregate.name().to_string(), city_id: aggregate.city_id().into() };
        let url = self.url("/stations");
        let resp = self.client.post(url).json(&dto).send().await.map_err(|e| RepositoryError::Db(e.into()))?;
        let SaveResultDTO { id } = resp.json().await.map_err(|e| RepositoryError::Db(e.into()))?;
        aggregate.set_id(StationId::from(id));
        Ok(StationId::from(id))
    }
}

#[async_trait]
impl StationRepository for StationRepositoryHttpImpl {
    async fn load(&self) -> Result<Vec<Station>, RepositoryError> {
        let url = self.url("/stations");
        let list: Vec<StationDTO> = self.client.get(url).send().await.map_err(|e| RepositoryError::Db(e.into()))?
            .json().await.map_err(|e| RepositoryError::Db(e.into()))?;
        Ok(list.into_iter().map(|d| Station::new(d.id.map(Into::into), d.name, CityId::from(d.city_id))).collect())
    }

    async fn find_by_city(&self, city_id: CityId) -> Result<Vec<Station>, RepositoryError> {
        let url = self.url(&format!("/stations?cityId={}", u64::from(city_id)));
        let list: Vec<StationDTO> = self.client.get(url).send().await.map_err(|e| RepositoryError::Db(e.into()))?
            .json().await.map_err(|e| RepositoryError::Db(e.into()))?;
        Ok(list.into_iter().map(|d| Station::new(d.id.map(Into::into), d.name, CityId::from(d.city_id))).collect())
    }

    async fn find_by_name(&self, station_name: &str) -> Result<Option<Station>, RepositoryError> {
        let url = self.url(&format!("/stations?name={}", urlencoding::encode(station_name)));
        let list: Vec<StationDTO> = self.client.get(url).send().await.map_err(|e| RepositoryError::Db(e.into()))?
            .json().await.map_err(|e| RepositoryError::Db(e.into()))?;
        Ok(list.into_iter().next().map(|d| Station::new(d.id.map(Into::into), d.name, CityId::from(d.city_id))))
    }

    async fn save_raw(&self, _station_data: shared::data::StationData) -> Result<(), RepositoryError> { Ok(()) }
}
