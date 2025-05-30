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
