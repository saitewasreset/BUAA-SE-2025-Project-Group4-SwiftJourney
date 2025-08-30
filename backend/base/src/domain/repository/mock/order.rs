#![cfg(test)]

use crate::domain::model::hotel::HotelId;
use crate::domain::model::order::{
    DishOrder, HotelOrder, Order, OrderId, TakeawayOrder, TrainOrder,
};
use crate::domain::model::train_schedule::TrainScheduleId;
use crate::domain::model::user::UserId;
use crate::domain::repository::order::{
    DishOrderRelatedData, HotelOrderRelatedData, OrderRepository, RouteInfo,
    TakeawayOrderRelatedData, TrainOrderRelatedData,
};
use crate::domain::RepositoryError;
use async_trait::async_trait;
use chrono::NaiveDate;
use mockall::mock;
use uuid::Uuid;

mock! {
    pub OrderRepository {}

    #[async_trait]
    impl OrderRepository for OrderRepository {
        async fn find_train_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<TrainOrder>, RepositoryError>;
    async fn find_hotel_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<HotelOrder>, RepositoryError>;

    async fn find_hotel_order_by_userid(
        &self,
        user_id: UserId,
        hotel_id: HotelId,
    ) -> Result<Vec<HotelOrder>, RepositoryError>;

    async fn find_dish_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<DishOrder>, RepositoryError>;

    async fn find_takeaway_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<TakeawayOrder>, RepositoryError>;

    async fn load_all_active_orders(&self) -> Result<Vec<Box<dyn Order>>, RepositoryError>;

    async fn update(&self, order: Box<dyn Order>) -> Result<(), RepositoryError>;

    async fn get_route_info_train_order(
        &self,
        order_id: OrderId,
        train_schedule_id: TrainScheduleId,
    ) -> Result<(NaiveDate, Vec<RouteInfo>), RepositoryError>;
    async fn get_route_info_takeaway_order(
        &self,
        order_id: OrderId,
        train_order_id: OrderId,
    ) -> Result<(NaiveDate, Vec<RouteInfo>), RepositoryError>;
    async fn get_train_order_related_data(
        &self,
        order_id: OrderId,
        train_schedule_id: TrainScheduleId,
        tz_offset_hour: i32,
    ) -> Result<TrainOrderRelatedData, RepositoryError>;

    async fn get_hotel_order_related_data(
        &self,
        order_id: OrderId,
    ) -> Result<HotelOrderRelatedData, RepositoryError>;

    async fn get_dish_order_related_data(
        &self,
        order_id: OrderId,
        tz_offset_hour: i32,
    ) -> Result<DishOrderRelatedData, RepositoryError>;

    async fn get_takeaway_order_related_data(
        &self,
        order_id: OrderId,
        train_order_id: OrderId,
        tz_offset_hour: i32,
    ) -> Result<TakeawayOrderRelatedData, RepositoryError>;

    async fn verify_train_order(
        &self,
        user_id: UserId,
        train_number: String,
        origin_departure_date: NaiveDate,
        origin_departure_time_second: i32,
    ) -> Result<bool, RepositoryError>;
    }
}

pub fn mock_order_repository() -> MockOrderRepository {
    MockOrderRepository::new()
}
