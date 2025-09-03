//! # Order DTOs and Serialization Logic
//!
//! This module provides the necessary structures and functions to serialize and deserialize
//! the `Order` trait object. It uses a tagged enum `OrderDTO` to handle polymorphism.
//! The DTO structs are only for internally use, so we assume that they are always valid.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfoDTO {
    pub transaction_id: Uuid,
    pub amount: f64,
    pub status: String,
}

use crate::Verified;
use crate::domain::Identifiable;
use crate::domain::model::dish::DishId;
use crate::domain::model::hotel::{HotelDateRange, HotelId, HotelRoomTypeId};
use crate::domain::model::order::{
    BaseOrder, DishOrder, HotelOrder, Order, OrderId, OrderStatus, OrderTimeInfo, PaymentInfo,
    TakeawayOrder, TrainOrder,
};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::personal_info::PreferredSeatLocation;
use crate::domain::model::station::StationId;
use crate::domain::model::takeaway::TakeawayDishId;
use crate::domain::model::train::{SeatType, SeatTypeId, SeatTypeName};
use crate::domain::model::train_schedule::{Seat, StationRange};
use crate::domain::model::train_schedule::{SeatId, SeatLocationInfo, SeatStatus, TrainScheduleId};
use crate::domain::model::transaction::TransactionId;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::any::Any;
use uuid::Uuid;
// Helper DTOs for nested structures

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalOrderTimeInfoDTO {
    pub create_time: DateTime<Utc>,
    pub active_time: DateTime<Utc>,
    pub complete_time: DateTime<Utc>,
}

impl From<OrderTimeInfo> for InternalOrderTimeInfoDTO {
    fn from(info: OrderTimeInfo) -> Self {
        Self {
            create_time: info.create_time().into(),
            active_time: info.active_time().into(),
            complete_time: info.complete_time().into(),
        }
    }
}

impl From<InternalOrderTimeInfoDTO> for OrderTimeInfo {
    fn from(dto: InternalOrderTimeInfoDTO) -> Self {
        Self::new(
            dto.create_time.into(),
            dto.active_time.into(),
            dto.complete_time.into(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternalPaymentInfoDTO {
    pub pay_transaction_id: Option<u64>,
    pub refund_transaction_id: Option<u64>,
}

impl From<PaymentInfo> for InternalPaymentInfoDTO {
    fn from(info: PaymentInfo) -> Self {
        Self {
            pay_transaction_id: info.pay_transaction_id().map(|id| id.into()),
            refund_transaction_id: info.refund_transaction_id().map(|id| id.into()),
        }
    }
}

impl From<InternalPaymentInfoDTO> for PaymentInfo {
    fn from(dto: InternalPaymentInfoDTO) -> Self {
        Self::new(
            dto.pay_transaction_id.map(TransactionId::from),
            dto.refund_transaction_id.map(TransactionId::from),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternalSeatTypeDTO {
    pub seat_type_id: u64,
    pub type_name: String,
    pub capacity: u32,
    pub price: Decimal,
}

impl From<SeatType> for InternalSeatTypeDTO {
    fn from(value: SeatType) -> Self {
        Self {
            seat_type_id: value.get_id().unwrap().into(),
            type_name: value.name().to_string(),
            capacity: value.capacity(),
            price: value.unit_price(),
        }
    }
}

impl From<InternalSeatTypeDTO> for SeatType {
    fn from(value: InternalSeatTypeDTO) -> Self {
        Self::new(
            Some(SeatTypeId::from(value.seat_type_id)),
            SeatTypeName::from_unchecked(value.type_name),
            value.capacity,
            value.price,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternalSeatDTO {
    pub id: u64,
    pub seat_type: InternalSeatTypeDTO,
    pub info: InternalSeatLocationInfoDTO,
    pub status: InternalSeatStatusDTO,
}

impl From<Seat> for InternalSeatDTO {
    fn from(seat: Seat) -> Self {
        Self {
            id: seat.get_id().unwrap().into(),
            seat_type: seat.seat_type().clone().into(),
            info: seat.location_info().into(),
            status: seat.status().into(),
        }
    }
}

impl From<InternalSeatDTO> for Seat {
    fn from(value: InternalSeatDTO) -> Self {
        Self::new(
            SeatId::from(value.id),
            value.seat_type.into(),
            value.info.into(),
            value.status.into(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternalSeatLocationInfoDTO {
    pub carriage: i32,  // 车厢号(如3)
    pub row: i32,       // 排数(如11)
    pub location: char, // 位置标记(如'A')
}

impl From<SeatLocationInfo> for InternalSeatLocationInfoDTO {
    fn from(value: SeatLocationInfo) -> Self {
        Self {
            carriage: value.carriage,
            row: value.row,
            location: value.location,
        }
    }
}

impl From<InternalSeatLocationInfoDTO> for SeatLocationInfo {
    fn from(value: InternalSeatLocationInfoDTO) -> Self {
        Self {
            carriage: value.carriage,
            row: value.row,
            location: value.location,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InternalSeatStatusDTO {
    Available,
    Occupied,
}

impl From<SeatStatus> for InternalSeatStatusDTO {
    fn from(value: SeatStatus) -> Self {
        match value {
            SeatStatus::Available => Self::Available,
            SeatStatus::Occupied => Self::Occupied,
        }
    }
}

impl From<InternalSeatStatusDTO> for SeatStatus {
    fn from(value: InternalSeatStatusDTO) -> Self {
        match value {
            InternalSeatStatusDTO::Available => Self::Available,
            InternalSeatStatusDTO::Occupied => Self::Occupied,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternalStationRangeDTO {
    pub start_station_id: u64,
    pub end_station_id: u64,
}

impl From<StationRange<Verified>> for InternalStationRangeDTO {
    fn from(range: StationRange<Verified>) -> Self {
        Self {
            start_station_id: range.get_from_station_id().into(),
            end_station_id: range.get_to_station_id().into(),
        }
    }
}

impl From<InternalStationRangeDTO> for StationRange<Verified> {
    fn from(value: InternalStationRangeDTO) -> Self {
        Self::from_unchecked(
            StationId::from(value.start_station_id),
            StationId::from(value.end_station_id),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternalHotelDateRangeDTO {
    pub begin_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl From<HotelDateRange> for InternalHotelDateRangeDTO {
    fn from(range: HotelDateRange) -> Self {
        Self {
            begin_date: range.begin_date(),
            end_date: range.end_date(),
        }
    }
}

impl From<InternalHotelDateRangeDTO> for HotelDateRange {
    fn from(dto: InternalHotelDateRangeDTO) -> Self {
        Self::new(dto.begin_date, dto.end_date).expect("Invalid internal hotel date range")
    }
}

// Base Order DTO

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalBaseOrderDTO {
    pub order_id: u64,
    pub uuid: Uuid,
    pub order_status: OrderStatus,
    pub order_time_info: InternalOrderTimeInfoDTO,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub payment_info: InternalPaymentInfoDTO,
    pub personal_info_id: u64,
}

impl From<BaseOrder> for InternalBaseOrderDTO {
    fn from(base: BaseOrder) -> Self {
        Self {
            order_id: base
                .order_id
                .expect("Base Order for transmission internally should have an id")
                .into(),
            uuid: base.uuid,
            order_status: base.order_status,
            order_time_info: base.order_time_info.into(),
            unit_price: base.unit_price,
            amount: base.amount,
            payment_info: base.payment_info.into(),
            personal_info_id: base.personal_info_id.into(),
        }
    }
}

impl From<InternalBaseOrderDTO> for BaseOrder {
    fn from(dto: InternalBaseOrderDTO) -> Self {
        Self::new(
            Some(OrderId::from(dto.order_id)),
            dto.uuid,
            dto.order_status,
            dto.order_time_info.into(),
            dto.unit_price,
            dto.amount,
            dto.payment_info.into(),
            PersonalInfoId::from(dto.personal_info_id),
        )
    }
}

// Main Order DTOs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InternalPreferredSeatLocationDTO {
    A,
    B,
    C,
    D,
    F,
}

impl From<PreferredSeatLocation> for InternalPreferredSeatLocationDTO {
    fn from(value: PreferredSeatLocation) -> Self {
        match value {
            PreferredSeatLocation::A => Self::A,
            PreferredSeatLocation::B => Self::B,
            PreferredSeatLocation::C => Self::C,
            PreferredSeatLocation::D => Self::D,
            PreferredSeatLocation::F => Self::F,
        }
    }
}

impl From<InternalPreferredSeatLocationDTO> for PreferredSeatLocation {
    fn from(value: InternalPreferredSeatLocationDTO) -> Self {
        match value {
            InternalPreferredSeatLocationDTO::A => Self::A,
            InternalPreferredSeatLocationDTO::B => Self::B,
            InternalPreferredSeatLocationDTO::C => Self::C,
            InternalPreferredSeatLocationDTO::D => Self::D,
            InternalPreferredSeatLocationDTO::F => Self::F,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalTrainOrderDTO {
    pub base: InternalBaseOrderDTO,
    pub train_schedule_id: u64,
    pub seat: Option<InternalSeatDTO>,
    pub order_seat_type_name: String,
    pub preferred_seat_location: Option<InternalPreferredSeatLocationDTO>,
    pub station_range: InternalStationRangeDTO,
}

impl From<TrainOrder> for InternalTrainOrderDTO {
    fn from(order: TrainOrder) -> Self {
        Self {
            train_schedule_id: order.train_schedule_id().into(),
            seat: order.seat().clone().map(Into::into),
            order_seat_type_name: order.order_seat_type_name().to_string(),
            preferred_seat_location: order.preferred_seat_location().map(|x| x.into()),
            station_range: order.station_range().into(),
            base: order.base().clone().into(),
        }
    }
}

impl From<InternalTrainOrderDTO> for TrainOrder {
    fn from(dto: InternalTrainOrderDTO) -> Self {
        let base_order = dto.base.into();
        let station_range = StationRange::from_unchecked(
            dto.station_range.start_station_id.into(),
            dto.station_range.end_station_id.into(),
        );

        Self::new(
            base_order,
            TrainScheduleId::from(dto.train_schedule_id),
            dto.seat.map(Into::into),
            SeatTypeName::from_unchecked(dto.order_seat_type_name),
            dto.preferred_seat_location.map(|x| x.into()),
            station_range,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalHotelOrderDTO {
    pub base: InternalBaseOrderDTO,
    pub hotel_id: u64,
    pub room_id: u64,
    pub booking_date_range: InternalHotelDateRangeDTO,
}

impl From<HotelOrder> for InternalHotelOrderDTO {
    fn from(order: HotelOrder) -> Self {
        Self {
            hotel_id: order.hotel_id().into(),
            room_id: order.room_id().into(),
            booking_date_range: order.booking_date_range().into(),
            base: order.base().clone().into(),
        }
    }
}

impl From<InternalHotelOrderDTO> for HotelOrder {
    fn from(dto: InternalHotelOrderDTO) -> Self {
        let base_order = dto.base.into();
        Self::new(
            base_order,
            HotelId::from(dto.hotel_id),
            HotelRoomTypeId::from(dto.room_id),
            dto.booking_date_range.into(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalDishOrderDTO {
    pub base: InternalBaseOrderDTO,
    pub train_order_id: u64,
    pub dish_id: u64,
    pub unit_price: Decimal,
    pub amount: Decimal,
}

impl From<DishOrder> for InternalDishOrderDTO {
    fn from(order: DishOrder) -> Self {
        Self {
            train_order_id: order.train_order_id().into(),
            dish_id: order.dish_id().into(),
            unit_price: order.unit_price(),
            amount: order.amount(),
            base: order.base().clone().into(),
        }
    }
}

impl From<InternalDishOrderDTO> for DishOrder {
    fn from(dto: InternalDishOrderDTO) -> Self {
        let base_order = dto.base.into();
        Self::new(
            base_order,
            OrderId::from(dto.train_order_id),
            DishId::from(dto.dish_id),
            dto.unit_price,
            dto.amount,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalTakeawayOrderDTO {
    pub base: InternalBaseOrderDTO,
    pub train_order_id: u64,
    pub takeaway_dish_id: u64,
    pub unit_price: Decimal,
    pub amount: Decimal,
}

impl From<TakeawayOrder> for InternalTakeawayOrderDTO {
    fn from(order: TakeawayOrder) -> Self {
        Self {
            train_order_id: order.train_order_id().into(),
            takeaway_dish_id: order.takeaway_dish_id().into(),
            unit_price: order.unit_price(),
            amount: order.amount(),
            base: order.base().clone().into(),
        }
    }
}

impl From<InternalTakeawayOrderDTO> for TakeawayOrder {
    fn from(dto: InternalTakeawayOrderDTO) -> Self {
        let base_order = dto.base.into();
        Self::new(
            base_order,
            OrderId::from(dto.train_order_id),
            TakeawayDishId::from(dto.takeaway_dish_id),
            dto.unit_price,
            dto.amount,
        )
    }
}

// Wrapper Enum for Polymorphism

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum InternalOrderDTO {
    Train(InternalTrainOrderDTO),
    Hotel(InternalHotelOrderDTO),
    Dish(InternalDishOrderDTO),
    Takeaway(InternalTakeawayOrderDTO),
}

impl From<&dyn Order> for InternalOrderDTO {
    fn from(order: &dyn Order) -> Self {
        let order_any = order as &dyn Any;
        if let Some(train_order) = order_any.downcast_ref::<TrainOrder>() {
            InternalOrderDTO::Train(train_order.clone().into())
        } else if let Some(hotel_order) = order_any.downcast_ref::<HotelOrder>() {
            InternalOrderDTO::Hotel(hotel_order.clone().into())
        } else if let Some(dish_order) = order_any.downcast_ref::<DishOrder>() {
            InternalOrderDTO::Dish(dish_order.clone().into())
        } else if let Some(takeaway_order) = order_any.downcast_ref::<TakeawayOrder>() {
            InternalOrderDTO::Takeaway(takeaway_order.clone().into())
        } else {
            panic!("Error while downcasting order to internal order dto, unknown order type")
        }
    }
}

impl From<InternalOrderDTO> for Box<dyn Order> {
    fn from(dto: InternalOrderDTO) -> Self {
        match dto {
            InternalOrderDTO::Train(train_dto) => {
                let order: TrainOrder = train_dto.into();
                Box::new(order)
            }
            InternalOrderDTO::Hotel(hotel_dto) => {
                let order: HotelOrder = hotel_dto.into();
                Box::new(order)
            }
            InternalOrderDTO::Dish(dish_dto) => {
                let order: DishOrder = dish_dto.into();
                Box::new(order)
            }
            InternalOrderDTO::Takeaway(takeaway_dto) => {
                let order: TakeawayOrder = takeaway_dto.into();
                Box::new(order)
            }
        }
    }
}

/// Serializes a boxed Order trait object into a JSON string.
pub fn serialize_order(order: &dyn Order) -> serde_json::Value {
    let dto = InternalOrderDTO::from(order);
    serde_json::to_value(&dto).unwrap()
}

/// Deserializes a JSON string into a boxed Order trait object.
pub fn deserialize_order(json_bytes: &[u8]) -> Result<Box<dyn Order>> {
    let dto: InternalOrderDTO = serde_json::from_slice(json_bytes)?;

    Ok(dto.into())
}
