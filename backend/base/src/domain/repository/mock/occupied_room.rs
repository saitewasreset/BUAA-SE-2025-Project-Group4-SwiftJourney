#![cfg(test)]

use crate::domain::model::hotel::{HotelDateRange, HotelId, OccupiedRoom, OccupiedRoomId};
use crate::domain::repository::occupied_room::OccupiedRoomRepository;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

mock! {
    pub OccupiedRoomRepository {}

    #[async_trait]
    impl OccupiedRoomRepository for OccupiedRoomRepository {
        async fn find_by_date_range(
        &self,
        hotel_id: HotelId,
        booking_date_range: HotelDateRange,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError>;

    async fn find_possible_occupied_range(
        &self,
        hotel_id: HotelId,
        booking_date_range: HotelDateRange,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError>;

    async fn save_count(
        &self,
        occupied_room: &OccupiedRoom,
        count: i32,
    ) -> Result<(), RepositoryError>;

    async fn find_by_order_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError>;

    async fn remove_many(
        &self,
        occupied_room_list: Vec<OccupiedRoom>,
    ) -> Result<(), RepositoryError>;
    }

    #[async_trait]
    impl Repository<OccupiedRoom> for OccupiedRoomRepository {
        async fn find(&self, id: OccupiedRoomId) -> Result<Option<OccupiedRoom>, RepositoryError>;
        async fn remove(&self, aggregate: OccupiedRoom) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut OccupiedRoom) -> Result<OccupiedRoomId, RepositoryError>;
    }
}

pub fn mock_occupied_room_repository() -> MockOccupiedRoomRepository {
    MockOccupiedRoomRepository::new()
}
