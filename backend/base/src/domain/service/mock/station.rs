#![cfg(test)]

use async_trait::async_trait;

use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::service::station::StationService;
use crate::domain::service::station::StationServiceError;

use mockall::mock;

mock! {
    pub StationService {}

    #[async_trait]
    impl StationService for StationService {
        async fn get_stations(&self) -> Result<Vec<Station>, StationServiceError>;

        async fn get_station_by_city(
            &self,
            city_id: CityId
        ) -> Result<Vec<Station>, StationServiceError>;

        async fn get_station_by_name(
            &self,
            station_name: String
        ) -> Result<Option<Station>, StationServiceError>;

        async fn add_station(
            &self,
            station_name: String,
            city_name: String
        ) -> Result<StationId, StationServiceError>;

        async fn modify_station(
            &self,
            station_id: StationId,
            station_name: String,
            city_name: String
        ) -> Result<(), StationServiceError>;

        async fn delete_station(
            &self,
            station: Station
        ) -> Result<(), StationServiceError>;

        async fn get_station_by_city_name(
            &self,
            city_name: &str
        ) -> Result<Vec<Station>, StationServiceError>;

        async fn station_pairs_by_city(
            &self,
            from_city: &str,
            to_city: &str
        ) -> Result<Vec<(StationId, StationId)>, StationServiceError>;
    }
}

// Helper 函数，方便测试中获取 Arc<MockStationService>
pub fn mock_station_service() -> MockStationService {
    MockStationService::new()
}
