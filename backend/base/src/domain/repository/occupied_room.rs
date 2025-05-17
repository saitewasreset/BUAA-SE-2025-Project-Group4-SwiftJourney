use crate::domain::model::hotel::{HotelDateRange, HotelId, OccupiedRoom};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OccupiedRoomRepository: Repository<OccupiedRoom> {
    // 闭区间：[begin_date, end_date]
    async fn find_by_date_range(
        &self,
        hotel_id: HotelId,
        booking_date_range: HotelDateRange,
    ) -> Result<Vec<OccupiedRoom>, RepositoryError>;

    // 返回满足如下条件的预订记录：
    // end_date > HotelDateRange.begin_date
    // begin_date < HotelDateRange.end_date
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
