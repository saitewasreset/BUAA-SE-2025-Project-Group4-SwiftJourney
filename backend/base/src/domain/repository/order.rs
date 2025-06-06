use crate::domain::RepositoryError;
use crate::domain::model::hotel::HotelId;
use crate::domain::model::order::{
    DishOrder, HotelOrder, Order, OrderId, TakeawayOrder, TrainOrder,
};
use crate::domain::model::train_schedule::TrainScheduleId;
use crate::domain::model::user::UserId;
use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::FromQueryResult;
use uuid::Uuid;
/*
c

SELECT route.order, route.arrival_time, route.departure_time, station.name FROM train_order
    INNER JOIN train_schedule
        ON train_schedule.id = train_order.train_schedule_id
    INNER JOIN route
        ON route.line_id = train_schedule.line_id
    INNER JOIN station
        ON station.id = route.station_id
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
pub struct RouteInfo {
    pub order: i32,
    /// 车次到达站点时间相对当日00:00:00的秒数，不是相对发车时间
    pub arrival_time: i32,
    /// 车次离开站点时间相对当日00:00:00的秒数，不是相对发车时间
    pub departure_time: i32,
    pub station_id: i32,
    pub station_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
pub struct TrainOrderRelatedData {
    /// 车次号
    pub train_number: String,
    pub departure_station: String,
    pub terminal_station: String,
    pub departure_time: String,
    pub terminal_time: String,
    /// 旅客姓名
    pub name: String,
}

/*
SELECT hotel.name, hotel.uuid, person_info.name, hotel_room_type.type_name FROM hotel_order
    INNER JOIN hotel
        ON hotel.id = hotel_order.hotel_id
    INNER JOIN person_info
        ON person_info.id = hotel_order.person_info_id
    INNER JOIN hotel_room_type
                ON hotel_room_type.id = hotel_order.hotel_room_type_id
    WHERE hotel_order.id = ?;
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
pub struct HotelOrderRelatedData {
    pub hotel_name: String,
    pub hotel_id: Uuid,
    pub name: String,
    pub room_type: String,
}

/*
SELECT train.number, route.departure_time, person_info.name, dish.name, dish.time FROM dish_order
    INNER JOIN train_order
        ON train_order.id = dish_order.train_order_id
    INNER JOIN train_schedule
        ON train_schedule.id = train_order.train_schedule_id
    INNER JOIN train
        ON train.id = train_schedule.train_id
    INNER JOIN route
        ON route.line_id = train_schedule.line_id
    INNER JOIN person_info
        ON person_info.id = dish_order.person_info_id
    INNER JOIN dish
        ON dish.id = dish_order.dish_id
    WHERE dish_order.id = ? AND route.order = 0;
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
pub struct DishOrderRelatedData {
    /// 车次号
    pub train_number: String,
    /// 离开起始站日期时间
    pub departure_time: String,
    /// 点餐人姓名
    pub name: String,
    /// 餐品名称
    pub dish_name: String,
    /// 用餐时间
    pub dish_time: String,
}

/*
SELECT train.number, route.departure_time, station.id, station.name, takeaway_shop.name, person_info.name, takeaway_dish.name FROM takeaway_order
    INNER JOIN train_order
        ON train_order.id = takeaway_order.train_order_id
    INNER JOIN train_schedule
        ON train_schedule.id = train_order.train_schedule_id
    INNER JOIN train
        ON train.id = train_schedule.train_id
    INNER JOIN route
        ON route.line_id = train_schedule.line_id
    INNER JOIN person_info
        ON person_info.id = takeaway_order.person_info_id
    INNER JOIN takeaway_dish
        ON takeaway_dish.id = takeaway_order.takeaway_dish_id
    INNER JOIN takeaway_shop
        ON takeaway_dish.takeaway_shop_id = takeaway_shop.id
    INNER JOIN station
        ON takeaway_shop.station_id = station.id
    WHERE takeaway_order.id = ? AND route.order = 0;

SELECT route.order, route.arrival_time, route.departure_time, station.name FROM takeaway_order
    INNER JOIN train_order
        ON takeaway_order.train_order_id = train_order.id
    INNER JOIN train_schedule
        ON train_schedule.id = train_order.train_schedule_id
    INNER JOIN route
        ON route.line_id = train_schedule.line_id
    INNER JOIN station
        ON station.id = route.station_id
    WHERE takeaway_order.id = ? ORDER BY route.order;
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
pub struct TakeawayOrderRelatedData {
    pub train_number: String,
    /// 离开起始站日期时间
    pub departure_time: String,
    pub station: String,
    pub shop_name: String,
    pub name: String,
    pub takeaway_name: String,
    pub dish_time: String,
}

#[async_trait]
pub trait OrderRepository: 'static + Send + Sync {
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
}
