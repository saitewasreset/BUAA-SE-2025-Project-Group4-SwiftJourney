use crate::application::service::internal::{GeoInternalService, GeoInternalServiceError};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::station::StationRepository;
use async_trait::async_trait;
use shared::MicroService;
use shared::data::{CityData, StationDataItem};
use shared::domain::Identifiable;
use shared::event::queue::EventService;
use shared::event::{CityUpdatedEvent, EventPackage, StationUpdatedEvent};
use shared::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use shared::internal::geo::dto::{
    CityDTO, CityInfoDTO, CityStationInfoDTO, DbCityDTO, DbStationDTO, StationDTO,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

pub struct GeoInternalServiceImpl<CR, SR, ES>
where
    CR: CityRepository,
    SR: StationRepository,
    ES: EventService,
{
    city_repository: Arc<CR>,
    station_repository: Arc<SR>,
    event_service: Arc<ES>,
}

impl<CR, SR, ES> GeoInternalServiceImpl<CR, SR, ES>
where
    CR: CityRepository,
    SR: StationRepository,
    ES: EventService,
{
    pub fn new(
        city_repository: Arc<CR>,
        station_repository: Arc<SR>,
        event_service: Arc<ES>,
    ) -> Self {
        Self {
            city_repository,
            station_repository,
            event_service,
        }
    }
}

#[async_trait]
impl<CR, SR, ES> GeoInternalService for GeoInternalServiceImpl<CR, SR, ES>
where
    CR: CityRepository,
    SR: StationRepository,
    ES: EventService,
{
    async fn db_get_cities(&self) -> Result<Vec<DbCityDTO>, GeoInternalServiceError> {
        Ok(self
            .city_repository
            .load_all_raw()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn db_get_stations(&self) -> Result<Vec<DbStationDTO>, GeoInternalServiceError> {
        Ok(self
            .station_repository
            .load_all_raw()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn get_cities(&self) -> Result<Vec<CityDTO>, GeoInternalServiceError> {
        Ok(self
            .city_repository
            .load()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, GeoInternalServiceError> {
        let station_list = self
            .station_repository
            .load()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?;

        let city_list = self
            .city_repository
            .load()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?;

        let city_id_to_city_name = city_list
            .into_iter()
            .map(|x| (x.get_id().unwrap(), x.name().to_string()))
            .collect::<HashMap<_, _>>();

        let mut city_to_station_list_map: HashMap<String, Vec<String>> = HashMap::new();

        for station in station_list {
            let city_name = city_id_to_city_name
                .get(&station.city_id())
                .cloned()
                .unwrap_or_default();

            let station_list = city_to_station_list_map.entry(city_name).or_default();

            station_list.push(station.name().to_string());
        }

        Ok(city_to_station_list_map)
    }

    async fn get_city_info_list(&self) -> Result<CityInfoDTO, GeoInternalServiceError> {
        let city_entity_list = self
            .city_repository
            .load()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?;

        let mut province_to_cities_name: HashMap<String, Vec<String>> = HashMap::new();

        for city in city_entity_list {
            let city_list = province_to_cities_name
                .entry(city.province().to_string())
                .or_default();

            city_list.push(city.name().to_string())
        }

        Ok(province_to_cities_name)
    }

    async fn get_stations(&self) -> Result<Vec<StationDTO>, GeoInternalServiceError> {
        Ok(self
            .station_repository
            .load()
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    async fn save_city_province_map(
        &self,
        cmd: SaveCityProvinceMapCommand,
    ) -> Result<(), GeoInternalServiceError> {
        let city_data: CityData = cmd.city_province_map;

        self.city_repository
            .save_raw(city_data)
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?;

        if let Err(e) = self
            .event_service
            .publish_event(EventPackage::new(MicroService::Geo, CityUpdatedEvent))
            .await
        {
            error!("Failed to publish city updated event: {:?}", e);
        }

        Ok(())
    }

    async fn save_station_city_map(
        &self,
        cmd: SaveStationCityMapCommand,
    ) -> Result<(), GeoInternalServiceError> {
        let station_data = cmd
            .station_city_map
            .into_iter()
            .map(|(station, city)| StationDataItem {
                name: station,
                city,
            })
            .collect();

        self.station_repository
            .save_raw(station_data)
            .await
            .map_err(|e| GeoInternalServiceError::RelatedServiceError(e.into()))?;

        if let Err(e) = self
            .event_service
            .publish_event(EventPackage::new(MicroService::Geo, StationUpdatedEvent))
            .await
        {
            error!("Failed to publish station updated event: {:?}", e);
        }

        Ok(())
    }
}
