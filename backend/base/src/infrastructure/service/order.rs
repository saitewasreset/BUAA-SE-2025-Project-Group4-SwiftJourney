use crate::domain::RepositoryError;
use crate::domain::model::order::{
    DishOrder, HotelOrder, Order, OrderStatus, TakeawayOrder, TrainOrder,
};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::order::OrderService;
use crate::domain::service::order::order_dto::*;

use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use std::any::{Any, TypeId};
use std::sync::Arc;

pub struct OrderServiceImpl<R>
where
    R: OrderRepository,
{
    order_repository: Arc<R>,
    tz_offset_hour: i32,
}

impl<R> OrderServiceImpl<R> where R: OrderRepository {}

// 公共订单字段宏，减少重复代码
macro_rules! base_order_fields {
    ($dto:ident, $order:expr, $order_type:expr) => {
        $dto {
            order_id: $order.order_id().to_string(),
            status: $order.order_status().to_string(),
            unit_price: $order.unit_price().to_f64().unwrap_or(0.0),
            amount: $order.amount().to_i32().unwrap_or(0),
            can_cancel: calculate_can_cancel($order),
            reason: get_reason($order.order_status(), $order.already_refund()),
            order_type: $order_type,
        }
    };
}

pub fn calculate_can_cancel(order: &dyn Order) -> bool {
    let status = order.order_status();
    !order.already_refund() && matches!(status, OrderStatus::Paid | OrderStatus::Ongoing)
}

pub fn get_reason(status: OrderStatus, already_refund: bool) -> Option<String> {
    if already_refund {
        return Some("已退款".into());
    }

    match status {
        OrderStatus::Active => Some("订单正在进行中".into()),
        OrderStatus::Completed => Some("订单已完成".into()),
        OrderStatus::Failed => Some("订单失败".into()),
        OrderStatus::Cancelled => Some("订单已取消".into()),
        _ => None,
    }
}

impl<R> OrderServiceImpl<R>
where
    R: OrderRepository,
{
    pub fn new(order_repository: Arc<R>, tz_offset_hour: i32) -> Self {
        Self {
            order_repository,
            tz_offset_hour,
        }
    }
}

#[async_trait]
impl<R> OrderService for OrderServiceImpl<R>
where
    R: OrderRepository,
{
    async fn convert_order_to_dto(
        &self,
        order: Box<dyn Order>,
    ) -> Result<OrderInfoDto, RepositoryError> {
        let type_id = (*order).type_id();

        let order_any = order.clone() as Box<dyn Any>;

        if type_id == TypeId::of::<TrainOrder>() {
            let train_order = order_any.downcast::<TrainOrder>().unwrap();

            let base = base_order_fields!(BaseOrderDto, order.as_ref(), "train".to_string());

            let related_info = self
                .order_repository
                .get_train_order_related_data(
                    train_order.order_id(),
                    train_order.train_schedule_id(),
                    self.tz_offset_hour,
                )
                .await?;

            let order_info_dto = TrainOrderDto {
                base,
                train_number: related_info.train_number,
                departure_station: related_info.departure_station,
                terminal_station: related_info.terminal_station,
                departure_time: related_info.departure_time,
                terminal_time: related_info.terminal_time,
                name: related_info.name,
                seat: SeatLocationInfoDTO {
                    carriage: train_order.seat().location_info().carriage,
                    row: train_order.seat().location_info().row,
                    location: String::from(train_order.seat().location_info().location),
                    type_name: train_order.seat().seat_type().name().to_string(),
                },
            };

            Ok(OrderInfoDto::Train(order_info_dto))
        } else if type_id == TypeId::of::<HotelOrder>() {
            let hotel_order = order_any.downcast::<HotelOrder>().unwrap();

            let base = base_order_fields!(BaseOrderDto, order.as_ref(), "hotel".to_string());

            let related_info = self
                .order_repository
                .get_hotel_order_related_data(hotel_order.order_id())
                .await?;

            let order_info_dto = HotelOrderDto {
                base,
                hotel_id: related_info.hotel_id.to_string(),
                name: related_info.name,
                room_type: related_info.room_type,
                begin_date: hotel_order.booking_date_range().begin_date().to_string(),
                hotel_name: related_info.hotel_name,
                end_date: hotel_order.booking_date_range().end_date().to_string(),
            };

            Ok(OrderInfoDto::Hotel(order_info_dto))
        } else if type_id == TypeId::of::<DishOrder>() {
            let dish_order = order_any.downcast::<DishOrder>().unwrap();

            let base = base_order_fields!(BaseOrderDto, order.as_ref(), "dish".to_string());

            let related_info = self
                .order_repository
                .get_dish_order_related_data(dish_order.order_id(), self.tz_offset_hour)
                .await?;

            let order_info_dto = DishOrderDto {
                base,
                train_number: related_info.train_number,
                departure_time: related_info.departure_time,
                dish_time: related_info.dish_time,
                name: related_info.name,
                dish_name: related_info.dish_name,
            };

            Ok(OrderInfoDto::Dish(order_info_dto))
        } else if type_id == TypeId::of::<TakeawayOrder>() {
            let takeaway_order = order_any.downcast::<TakeawayOrder>().unwrap();

            let base = base_order_fields!(BaseOrderDto, order.as_ref(), "takeaway".to_string());

            let related_info = self
                .order_repository
                .get_takeaway_order_related_data(
                    takeaway_order.order_id(),
                    takeaway_order.train_order_id(),
                    self.tz_offset_hour,
                )
                .await?;

            let order_info_dto = TakeawayOrderDto {
                base,
                train_number: related_info.train_number,
                departure_time: related_info.departure_time,
                station: related_info.station,
                dish_time: related_info.dish_time,
                shop_name: related_info.shop_name,
                name: related_info.name,
                takeaway_name: related_info.takeaway_name,
            };

            Ok(OrderInfoDto::Takeaway(order_info_dto))
        } else {
            panic!("Unknown order type")
        }
    }
}
