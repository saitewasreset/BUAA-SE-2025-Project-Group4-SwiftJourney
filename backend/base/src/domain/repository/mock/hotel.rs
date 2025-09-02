use crate::domain::model::city::CityId;
use crate::domain::model::hotel::{Hotel, HotelId};
use crate::domain::model::station::StationId;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::{Identifiable, Repository, RepositoryError};
use async_trait::async_trait;
use uuid::Uuid;

/// Mock 实现，只用内存保存一个 hotel
pub struct MockHotelRepository {
    pub hotel: Option<Hotel>,
    pub hotel_id: Option<HotelId>,
}

impl MockHotelRepository {
    pub fn new(hotel: Option<Hotel>, hotel_id: Option<HotelId>) -> Self {
        Self { hotel, hotel_id }
    }
}

#[async_trait]
impl Repository<Hotel> for MockHotelRepository {
    async fn find(&self, id: HotelId) -> Result<Option<Hotel>, RepositoryError> {
        // 只有 id 匹配 self.hotel_id 时才返回 Some(hotel)，否则返回 None
        if Some(id) == self.hotel_id {
            Ok(self.hotel.clone())
        } else {
            Ok(None)
        }
    }

    async fn remove(&self, _entity: Hotel) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn save(&self, _entity: &mut Hotel) -> Result<HotelId, RepositoryError> {
        Ok(_entity.get_id().unwrap())
    }
}

#[async_trait]
impl HotelRepository for MockHotelRepository {
    async fn get_id_by_uuid(&self, _uuid: Uuid) -> Result<Option<HotelId>, RepositoryError> {
        Ok(self.hotel_id)
    }

    async fn find_by_uuid(&self, _uuid: Uuid) -> Result<Option<Hotel>, RepositoryError> {
        Ok(self.hotel.clone())
    }

    async fn find_by_city(
        &self,
        _city_id: CityId,
        _name_pattern: Option<&str>,
    ) -> Result<Vec<Hotel>, RepositoryError> {
        Ok(self.hotel.clone().into_iter().collect())
    }

    async fn find_by_station(
        &self,
        _station_id: StationId,
        _name_pattern: Option<&str>,
    ) -> Result<Vec<Hotel>, RepositoryError> {
        Ok(self.hotel.clone().into_iter().collect())
    }
}

pub fn mock_hotel_repository() -> MockHotelRepository {
    MockHotelRepository::new(None, None)
}
