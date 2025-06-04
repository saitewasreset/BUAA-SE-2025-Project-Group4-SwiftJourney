use crate::domain::RepositoryError;
use crate::domain::model::order::Order;
use crate::domain::service::order::order_dto::OrderInfoDto;
use async_trait::async_trait;

pub mod order_dto {
    use serde::Serialize;
    // DTO结构体定义

    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct TransactionDataDto {
        pub transaction_id: String,
        pub status: String,
        pub create_time: String,
        pub pay_time: Option<String>,
        pub orders: Vec<OrderInfoDto>,
        pub amount: f64,
    }

    #[derive(Serialize, Clone)]
    #[serde(tag = "orderType", content = "data")]
    pub enum OrderInfoDto {
        Train(TrainOrderDto),
        Hotel(HotelOrderDto),
        Dish(DishOrderDto),
        Takeaway(TakeawayOrderDto),
    }

    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SeatLocationInfoDTO {
        pub carriage: i32,
        pub row: i32,
        pub location: String,
        #[serde(rename = "type")]
        pub type_name: String,
    }

    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct TrainOrderDto {
        // 公共字段
        #[serde(flatten)]
        pub base: BaseOrderDto,
        // 特有字段
        pub train_number: String,
        pub departure_station: String,
        pub terminal_station: String,
        pub departure_time: String,
        pub terminal_time: String,
        pub name: String,
        pub seat: SeatLocationInfoDTO,
    }

    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct HotelOrderDto {
        #[serde(flatten)]
        pub base: BaseOrderDto,
        pub hotel_name: String,
        pub hotel_id: String,
        pub name: String,
        pub room_type: String,
        pub begin_date: String,
        pub end_date: String,
    }

    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct DishOrderDto {
        #[serde(flatten)]
        pub base: BaseOrderDto,
        pub train_number: String,
        /// 离开起始站日期时间
        pub departure_time: String,
        pub dish_time: String,
        pub name: String,
        pub dish_name: String,
    }

    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct TakeawayOrderDto {
        #[serde(flatten)]
        pub base: BaseOrderDto,
        pub train_number: String,
        /// 离开起始站日期时间
        pub departure_time: String,
        pub station: String,
        pub dish_time: String,
        pub shop_name: String,
        pub name: String,
        pub takeaway_name: String,
    }

    #[derive(Serialize, Clone)]
    pub struct BaseOrderDto {
        pub order_id: String,
        pub status: String,
        pub unit_price: f64,
        pub amount: i32,
        pub can_cancel: bool,
        pub reason: Option<String>,
        pub order_type: String,
    }
}

#[async_trait]
pub trait OrderService: 'static + Send + Sync {
    async fn convert_order_to_dto(
        &self,
        order: Box<dyn Order>,
    ) -> Result<OrderInfoDto, RepositoryError>;
}
