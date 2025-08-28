use crate::application::service::geo::{CityInfoDTO, CityStationInfoDTO, GeoApplicationService};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::service::geo::GeoService;
use crate::domain::service::station::StationService;
use crate::domain::{DbId, Identifiable};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct GeoApplicationServiceImpl<G, S>
where
    G: GeoService + 'static + Send + Sync,
    S: StationService + 'static + Send + Sync,
{
    geo_service: Arc<G>,
    station_service: Arc<S>,
}

impl<G, S> GeoApplicationServiceImpl<G, S>
where
    G: GeoService + 'static + Send + Sync,
    S: StationService + 'static + Send + Sync,
{
    pub fn new(geo_service: Arc<G>, station_service: Arc<S>) -> Self {
        GeoApplicationServiceImpl {
            geo_service,
            station_service,
        }
    }
}

#[async_trait]
impl<G, S> GeoApplicationService for GeoApplicationServiceImpl<G, S>
where
    G: GeoService + 'static + Send + Sync,
    S: StationService + 'static + Send + Sync,
{
    #[instrument(skip(self))]
    async fn get_city_info(&self) -> Result<CityInfoDTO, Box<dyn ApplicationError>> {
        Ok(self
            .geo_service
            .get_city_map()
            .await
            .map_err(|e| {
                error!("Failed to get city map: {:?}", e);
                GeneralError::InternalServerError
            })?
            .into_iter()
            .map(|(province, city_list)| {
                let city_names = city_list
                    .into_iter()
                    .map(|city| city.name().to_string())
                    .collect();
                (province.to_string(), city_names)
            })
            .collect())
    }

    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, Box<dyn ApplicationError>> {
        let city_id_to_name = self
            .geo_service
            .get_city_map()
            .await
            .map_err(|e| {
                error!("Failed to get city map: {:?}", e);
                GeneralError::InternalServerError
            })?
            .into_values()
            .flatten()
            .map(|city| {
                let city_id = city
                    .get_id()
                    .expect("City loaded from database should have an ID")
                    .to_db_value();
                let city_name = city.name().to_string();
                (city_id, city_name)
            })
            .collect::<HashMap<_, _>>();

        let stations = self.station_service.get_stations().await.map_err(|e| {
            error!("Failed to get stations: {:?}", e);
            GeneralError::InternalServerError
        })?;

        let mut city_station_map: HashMap<String, Vec<String>> = HashMap::new();

        for station in stations {
            if let Some(city_name) = city_id_to_name.get(&station.city_id().to_db_value()) {
                city_station_map
                    .entry(city_name.to_string())
                    .or_default()
                    .push(station.name().to_string());
            } else {
                error!(
                    "Inconsistent: City ID {} not found city table",
                    station.city_id().to_db_value()
                );
            }
        }

        Ok(city_station_map)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RepositoryError;
    use crate::domain::model::city::{ City, CityId, CityName, ProvinceName };
    use crate::domain::service::geo::GeoServiceError;
    use crate::domain::model::station::{ Station, StationId };
    use crate::domain::service::station::StationServiceError;
    use crate::domain::service::ServiceError;
    use mockall::mock;
    use std::collections::HashMap;
    use std::sync::Arc;
    use anyhow::anyhow;

    // ----------- Mock 定义 -----------

    mock! {
        pub GeoService {}
        #[async_trait]
        impl GeoService for GeoService {
            async fn get_city_map(&self) -> Result<HashMap<ProvinceName, Vec<City>>, GeoServiceError>;
            async fn get_city_by_name(&self, name: &str) -> Result<Option<City>, GeoServiceError>;
            async fn add_city(&self, city: City) -> Result<CityId, GeoServiceError>;
            async fn remove_city(&self, city: City) -> Result<(), GeoServiceError>;
            async fn modify_city(&self, city_id: CityId, city_name: CityName, province: ProvinceName) -> Result<(), GeoServiceError>;
        }
    }

    mock! {
        pub StationService {}
        #[async_trait]
        impl StationService for StationService {
            async fn get_stations(&self) -> Result<Vec<Station>, StationServiceError>;
            async fn get_station_by_city(&self, city_id: CityId) -> Result<Vec<Station>, StationServiceError>;
            async fn get_station_by_name(&self, station_name: String) -> Result<Option<Station>, StationServiceError>;
            async fn add_station(&self, station_name: String, city_name: String) -> Result<StationId, StationServiceError>;
            async fn modify_station(&self, station_id: StationId, station_name: String, city_name: String) -> Result<(), StationServiceError>;
            async fn delete_station(&self, station: Station) -> Result<(), StationServiceError>;
            async fn get_station_by_city_name(&self, city_name: &str) -> Result<Vec<Station>, StationServiceError>;
            async fn station_pairs_by_city(&self, from_city: &str, to_city: &str) -> Result<Vec<(StationId, StationId)>, StationServiceError>;
        }
    }

    // ----------- 正向测试 -----------

    #[tokio::test]
    async fn test_get_city_info_success() {
        let mut geo_mock = MockGeoService::new();
        let station_mock = MockStationService::new();

        // 模拟返回城市
        let city = City::new(
            Some(1u64.into()),
            "Beijing".to_string().into(),
            "Beijing".to_string().into(),
        );
        geo_mock
            .expect_get_city_map()
            .returning(move || {
                Ok(HashMap::from([(
                    "BeijingProvince".to_string().into(),
                    vec![city.clone()],
                )]))
            });

        let service = GeoApplicationServiceImpl::new(Arc::new(geo_mock), Arc::new(station_mock));

        let result = service.get_city_info().await;
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert!(dto.contains_key("BeijingProvince"));
        assert_eq!(dto["BeijingProvince"], vec!["Beijing".to_string()]);
    }

    // ----------- 反向测试 (GeoService 出错) -----------
    #[tokio::test]
    async fn test_get_city_info_fail() {
        let mut geo_mock = MockGeoService::new();
        let station_mock = MockStationService::new();

        geo_mock
            .expect_get_city_map()
            .returning(|| Err(GeoServiceError::InfrastructureError(
                ServiceError::RepositoryError(RepositoryError::InconsistentState(anyhow!("geo error")))
            )));

        let service = GeoApplicationServiceImpl::new(Arc::new(geo_mock), Arc::new(station_mock));

        let result = service.get_city_info().await;
        assert!(result.is_err());
    }

    // ----------- 正向测试 get_city_station_info -----------
    #[tokio::test]
    async fn test_get_city_station_info_success() {
        let mut geo_mock = MockGeoService::new();
        let mut station_mock = MockStationService::new();

        // city map
        let city = City::new(
            Some(1u64.into()),
            "Beijing".to_string().into(),
            "Beijing".to_string().into(),
        );
        geo_mock
            .expect_get_city_map()
            .returning(move || {
                Ok(HashMap::from([("SomeProvince".to_string().into(), vec![city.clone()])]))
            });

        // stations
        let station = Station::new(
            Some(1u64.into()),
            "BeijingSouth".to_string(),
            1u64.into(),
        );
        station_mock
            .expect_get_stations()
            .returning(move || Ok(vec![station.clone()]));

        let service = GeoApplicationServiceImpl::new(Arc::new(geo_mock), Arc::new(station_mock));

        let result = service.get_city_station_info().await;
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert!(dto.contains_key("Beijing"));
        assert_eq!(dto["Beijing"], vec!["BeijingSouth".to_string()]);
    }

    // ----------- 反向测试 (StationService 出错) -----------
    #[tokio::test]
    async fn test_get_city_station_info_fail() {
        let mut geo_mock = MockGeoService::new();
        let mut station_mock = MockStationService::new();

        let city = City::new(
            Some(2u64.into()),
            "Shanghai".to_string().into(),
            "Shanghai".to_string().into(),
        );
        geo_mock
            .expect_get_city_map()
            .returning(move || {
                Ok(HashMap::from([("SomeProvince".to_string().into(), vec![city.clone()])]))
            });

        station_mock
            .expect_get_stations()
            .returning(|| Err(StationServiceError::InfrastructureError(
                ServiceError::RepositoryError(RepositoryError::InconsistentState(anyhow!("db down")))
            )));

        let service = GeoApplicationServiceImpl::new(Arc::new(geo_mock), Arc::new(station_mock));

        let result = service.get_city_station_info().await;
        assert!(result.is_err());
    }
}

