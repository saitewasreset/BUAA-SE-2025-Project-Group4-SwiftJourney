use crate::domain::model::order::{
    DishOrder, HotelOrder, Order, OrderStatus, TakeawayOrder, TrainOrder,
};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::order::order_dto::*;
use crate::domain::service::order::OrderService;
use crate::domain::RepositoryError;

use crate::domain::model::user::UserId;
use async_trait::async_trait;
use chrono::Timelike;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
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
            order_id: $order.uuid().to_string(),
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
    !order.already_refund() && matches!(status, OrderStatus::Ongoing)
}

pub fn get_reason(status: OrderStatus, already_refund: bool) -> Option<String> {
    if already_refund {
        return Some("已退款".into());
    }

    match status {
        OrderStatus::Unpaid => Some("订单未支付".into()),
        OrderStatus::Paid => Some("订单正在处理中".into()),
        OrderStatus::Ongoing => None,
        OrderStatus::Active => Some("订单正在进行中".into()),
        OrderStatus::Completed => Some("订单已完成".into()),
        OrderStatus::Failed => Some("订单失败".into()),
        OrderStatus::Cancelled => Some("订单已取消".into()),
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
                    train_order.order_id().expect("order should have id"),
                    train_order.train_schedule_id(),
                    self.tz_offset_hour,
                )
                .await?;

            let seat = train_order.seat();

            let order_info_dto = TrainOrderDto {
                base,
                train_number: related_info.train_number,
                departure_station: related_info.departure_station,
                arrival_station: related_info.arrival_station,
                departure_time: related_info.departure_time,
                arrival_time: related_info.arrival_time,

                origin_station: related_info.origin_station,
                terminal_station: related_info.terminal_station,
                origin_departure_time: related_info.origin_departure_time,
                terminal_arrival_time: related_info.terminal_arrival_time,

                name: related_info.name,
                seat: seat.as_ref().map(|seat| SeatLocationInfoDTO {
                    carriage: seat.location_info().carriage,
                    row: seat.location_info().row,
                    location: String::from(seat.location_info().location),
                    type_name: seat.seat_type().name().to_string(),
                }),
            };

            Ok(OrderInfoDto::Train(order_info_dto))
        } else if type_id == TypeId::of::<HotelOrder>() {
            let hotel_order = order_any.downcast::<HotelOrder>().unwrap();

            let base = base_order_fields!(BaseOrderDto, order.as_ref(), "hotel".to_string());

            let related_info = self
                .order_repository
                .get_hotel_order_related_data(hotel_order.order_id().expect("order should have id"))
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
                .get_dish_order_related_data(
                    dish_order.order_id().expect("order should have id"),
                    self.tz_offset_hour,
                )
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
                    takeaway_order.order_id().expect("order should have id"),
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

    async fn verify_train_order(
        &self,
        user_id: UserId,
        train_number: String,
        origin_departure_time: DateTimeWithTimeZone,
    ) -> Result<bool, RepositoryError> {
        self.order_repository
            .verify_train_order(
                user_id,
                train_number,
                origin_departure_time.date_naive(),
                origin_departure_time.time().num_seconds_from_midnight() as i32,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::hotel::HotelDateRange;
    use crate::domain::model::order::{BaseOrder, OrderTimeInfo, PaymentInfo};
    use crate::domain::model::personal_info::PreferredSeatLocation;
    use crate::domain::model::train::{SeatType, SeatTypeName};
    use crate::domain::model::train_schedule::{Seat, SeatLocationInfo, SeatStatus, StationRange};
    use crate::domain::model::transaction::Transaction;
    use crate::domain::repository::mock::order::MockOrderRepository;
    use crate::domain::repository::order::TrainOrderRelatedData;
    use anyhow::anyhow;
    use chrono::{DateTime, FixedOffset, NaiveDate};
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_service(repo: MockOrderRepository) -> OrderServiceImpl<MockOrderRepository> {
        OrderServiceImpl::new(Arc::new(repo), 8) // 假设 tz_offset_hour = 8
    }

    // ---------- convert_order_to_dto 测试 ----------
    #[tokio::test]
    async fn test_convert_order_to_dto_train_success() {
        let mut repo = MockOrderRepository::new();

        repo.expect_get_train_order_related_data()
            .returning(|_, _, _| {
                Ok(TrainOrderRelatedData {
                    train_number: "G123".to_string(),
                    departure_station: "Beijing".to_string(),
                    arrival_station: "Shanghai".to_string(),
                    departure_time: Default::default(),
                    arrival_time: Default::default(),
                    origin_station: "Beijing".to_string(),
                    terminal_station: "Shanghai".to_string(),
                    origin_departure_time: Default::default(),
                    terminal_arrival_time: Default::default(),
                    name: "Alice".to_string(),
                })
            });

        let service = make_service(repo);

        let order_uuid = Uuid::new_v4();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            order_uuid,
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let train_order = TrainOrder::new(
            base_order,
            1u64.into(),
            Some(Seat::new(
                1u64.into(),
                SeatType::new(
                    Some(1u64.into()),
                    SeatTypeName::from_unchecked("二等座".to_string()),
                    1,
                    Decimal::new(1000, 2),
                ),
                SeatLocationInfo {
                    carriage: 3,
                    row: 11,
                    location: 'A',
                },
                SeatStatus::Occupied,
            )),
            SeatTypeName::from_unchecked("二等座".to_string()),
            Some(PreferredSeatLocation::A),
            StationRange::from_unchecked(1u64.into(), 1u64.into()),
        );

        let order = Box::new(train_order);

        let dto = service.convert_order_to_dto(order).await.unwrap();
        match dto {
            OrderInfoDto::Train(train) => {
                assert_eq!(train.train_number, "G123");
                assert_eq!(train.departure_station, "Beijing");
            }
            _ => panic!("expected train order dto"),
        }
    }

    #[tokio::test]
    async fn test_convert_order_to_dto_train_fail() {
        let mut repo = MockOrderRepository::new();
        repo.expect_get_train_order_related_data()
            .returning(|_, _, _| Err(RepositoryError::Db(anyhow!("db error"))));

        let service = make_service(repo);

        let order_uuid = Uuid::new_v4();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            order_uuid,
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let train_order = TrainOrder::new(
            base_order,
            1u64.into(),
            Some(Seat::new(
                1u64.into(),
                SeatType::new(
                    Some(1u64.into()),
                    SeatTypeName::from_unchecked("二等座".to_string()),
                    1,
                    Decimal::new(1000, 2),
                ),
                SeatLocationInfo {
                    carriage: 3,
                    row: 11,
                    location: 'A',
                },
                SeatStatus::Occupied,
            )),
            SeatTypeName::from_unchecked("二等座".to_string()),
            Some(PreferredSeatLocation::A),
            StationRange::from_unchecked(1u64.into(), 1u64.into()),
        );

        let order = Box::new(train_order);

        let result = service.convert_order_to_dto(order).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_convert_order_to_dto_hotel_success() {
        let mut repo = MockOrderRepository::new();
        repo.expect_get_hotel_order_related_data().returning(|_| {
            Ok(crate::domain::repository::order::HotelOrderRelatedData {
                hotel_id: Uuid::new_v4(),
                name: "Alice".to_string(),
                room_type: "Deluxe".to_string(),
                hotel_name: "Hilton".to_string(),
            })
        });

        let service = make_service(repo);

        let order_uuid = Uuid::new_v4();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            order_uuid,
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let hotel_id = 1u64.into();
        let hotel_room_type_id = 1u64.into();

        let hotel_order = HotelOrder::new(
            base_order,
            hotel_id,
            hotel_room_type_id,
            HotelDateRange::new(
                NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
            )
            .unwrap(),
        );

        let order = Box::new(hotel_order);

        let dto = service.convert_order_to_dto(order).await.unwrap();
        match dto {
            OrderInfoDto::Hotel(hotel) => {
                assert_eq!(hotel.hotel_name, "Hilton");
            }
            _ => panic!("expected hotel order dto"),
        }
    }

    // ---------- verify_train_order 测试 ----------
    #[tokio::test]
    async fn test_verify_train_order_success() {
        let mut repo = MockOrderRepository::new();
        repo.expect_verify_train_order()
            .returning(|_, _, _, _| Ok(true));

        let service = make_service(repo);
        let user_id = 1u64.into();
        let dt: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-08-30T08:00:00+08:00").unwrap();

        let result = service
            .verify_train_order(user_id, "G123".to_string(), dt)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_verify_train_order_fail() {
        let mut repo = MockOrderRepository::new();
        repo.expect_verify_train_order()
            .returning(|_, _, _, _| Err(RepositoryError::Db(anyhow!("db error"))));

        let service = make_service(repo);
        let user_id = 1u64.into();
        let dt: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-08-30T08:00:00+08:00").unwrap();

        let result = service
            .verify_train_order(user_id, "G123".to_string(), dt)
            .await;
        assert!(result.is_err());
    }
}
