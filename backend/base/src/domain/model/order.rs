use crate::Verified;
use crate::domain::model::dish::DishId;
use crate::domain::model::hotel::{HotelDateRange, HotelId, HotelRoomId};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::takeaway::TakeawayDishId;
use crate::domain::model::train_schedule::{Seat, StationRange, TrainScheduleId};
use crate::domain::model::transaction::TransactionId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use dyn_clone::{DynClone, clone_trait_object};
use id_macro::define_id_type;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    /// 订单已经生成，还未支付
    Unpaid,
    /// 订单已经生成，且支付成功，此时订单将进入后端消息队列；系统处理完成后，根据实际情况进入“未出行”或“失败”状态（如是否有可用座位）
    Paid,
    /// 订单已被后端处理，且成功（例如，火车订票有符合条件的座位，订票成功），还没有出行
    Ongoing,
    /// 行程正在进行（例如，火车订票已经过始发站，还未到达终到站）
    Active,
    /// 行程已经完成（例如，火车订票已到达终到站）
    Completed,
    /// 订单已被后端处理，且失败（例如，火车订票没有符合条件的座位，订票失败）
    Failed,
    /// 订单被用户取消
    Cancelled,
}

impl TryFrom<&str> for OrderStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "unpaid" => OrderStatus::Unpaid,
            "paid" => OrderStatus::Paid,
            "ongoing" => OrderStatus::Ongoing,
            "active" => OrderStatus::Active,
            "completed" => OrderStatus::Completed,
            "failed" => OrderStatus::Failed,
            "cancelled" => OrderStatus::Failed,
            x => return Err(format!("Invalid order status: {}", x)),
        })
    }
}

impl From<OrderStatus> for &'static str {
    fn from(value: OrderStatus) -> Self {
        match value {
            OrderStatus::Unpaid => "unpaid",
            OrderStatus::Paid => "paid",
            OrderStatus::Ongoing => "ongoing",
            OrderStatus::Active => "active",
            OrderStatus::Completed => "completed",
            OrderStatus::Failed => "failed",
            OrderStatus::Cancelled => "cancelled",
        }
    }
}

impl From<&OrderStatus> for &'static str {
    fn from(value: &OrderStatus) -> Self {
        (*value).into()
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&OrderStatus as Into<&'static str>>::into(self))
    }
}

define_id_type!(Order);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderTimeInfo {
    create_time: DateTimeWithTimeZone,
    active_time: DateTimeWithTimeZone,
    complete_time: DateTimeWithTimeZone,
}

impl OrderTimeInfo {
    pub fn new(
        crate_time: DateTimeWithTimeZone,
        active_time: DateTimeWithTimeZone,
        complete_time: DateTimeWithTimeZone,
    ) -> Self {
        Self {
            create_time: crate_time,
            active_time,
            complete_time,
        }
    }

    pub fn crate_time(&self) -> DateTimeWithTimeZone {
        self.create_time
    }

    pub fn active_time(&self) -> DateTimeWithTimeZone {
        self.active_time
    }

    pub fn complete_time(&self) -> DateTimeWithTimeZone {
        self.complete_time
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaymentInfo {
    pay_transaction_id: Option<TransactionId>,
    refund_transaction_id: Option<TransactionId>,
}

impl PaymentInfo {
    pub fn new(
        pay_transaction_id: Option<TransactionId>,
        refund_transaction_id: Option<TransactionId>,
    ) -> Self {
        Self {
            pay_transaction_id,
            refund_transaction_id,
        }
    }

    pub fn pay_transaction_id(&self) -> Option<TransactionId> {
        self.pay_transaction_id
    }

    pub fn refund_transaction_id(&self) -> Option<TransactionId> {
        self.refund_transaction_id
    }
}

pub trait Order: DynClone + Debug + Send + Sync + 'static + Any {
    fn order_id(&self) -> OrderId;

    fn uuid(&self) -> Uuid;

    fn already_refund(&self) -> bool;

    fn order_status(&self) -> OrderStatus;
    fn order_time_info(&self) -> OrderTimeInfo;

    fn unit_price(&self) -> Decimal;

    fn amount(&self) -> Decimal;

    fn payment_info(&self) -> PaymentInfo;

    fn personal_info_id(&self) -> PersonalInfoId;
}

clone_trait_object!(Order);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BaseOrder {
    order_id: OrderId,
    uuid: Uuid,
    order_status: OrderStatus,
    order_time_info: OrderTimeInfo,
    unit_price: Decimal,
    amount: Decimal,
    payment_info: PaymentInfo,
    personal_info_id: PersonalInfoId,
}

impl BaseOrder {
    pub fn new(
        order_id: OrderId,
        uuid: Uuid,
        order_status: OrderStatus,
        order_time_info: OrderTimeInfo,
        unit_price: Decimal,
        amount: Decimal,
        payment_info: PaymentInfo,
        personal_info_id: PersonalInfoId,
    ) -> Self {
        Self {
            order_id,
            uuid,
            order_status,
            order_time_info,
            unit_price,
            amount,
            payment_info,
            personal_info_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrainOrder {
    base: BaseOrder,
    train_schedule_id: TrainScheduleId,
    seat: Seat,
    station_range: StationRange<Verified>,
}

impl TrainOrder {
    pub fn new(
        base_order: BaseOrder,
        train_schedule_id: TrainScheduleId,
        seat: Seat,
        station_range: StationRange<Verified>,
    ) -> Self {
        Self {
            base: base_order,
            train_schedule_id,
            seat,
            station_range,
        }
    }

    pub fn train_schedule_id(&self) -> TrainScheduleId {
        self.train_schedule_id
    }

    pub fn station_range(&self) -> StationRange<Verified> {
        self.station_range
    }

    pub fn seat(&self) -> &Seat {
        &self.seat
    }
}

impl Order for TrainOrder {
    fn order_id(&self) -> OrderId {
        self.base.order_id
    }

    fn uuid(&self) -> Uuid {
        self.base.uuid
    }

    fn already_refund(&self) -> bool {
        self.base.payment_info.refund_transaction_id.is_some()
    }

    fn order_status(&self) -> OrderStatus {
        self.base.order_status
    }

    fn order_time_info(&self) -> OrderTimeInfo {
        self.base.order_time_info
    }

    fn unit_price(&self) -> Decimal {
        self.base.unit_price
    }

    fn amount(&self) -> Decimal {
        self.base.amount
    }

    fn payment_info(&self) -> PaymentInfo {
        self.base.payment_info
    }

    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }
}

impl Identifiable for TrainOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        Some(self.base.order_id)
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = id
    }
}

impl Entity for TrainOrder {}
impl Aggregate for TrainOrder {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotelOrder {
    base: BaseOrder,
    hotel_id: HotelId,
    room_id: HotelRoomId,
    booking_date_range: HotelDateRange,
}

impl HotelOrder {
    pub fn new(
        base_order: BaseOrder,
        hotel_id: HotelId,
        room_id: HotelRoomId,
        booking_date_range: HotelDateRange,
    ) -> Self {
        Self {
            base: base_order,
            hotel_id,
            room_id,
            booking_date_range,
        }
    }

    pub fn hotel_id(&self) -> HotelId {
        self.hotel_id
    }

    pub fn room_id(&self) -> HotelRoomId {
        self.room_id
    }

    pub fn booking_date_range(&self) -> HotelDateRange {
        self.booking_date_range
    }
}

impl Identifiable for HotelOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        Some(self.base.order_id)
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = id
    }
}

impl Entity for HotelOrder {}
impl Aggregate for HotelOrder {}

impl Order for HotelOrder {
    fn order_id(&self) -> OrderId {
        self.base.order_id
    }

    fn uuid(&self) -> Uuid {
        self.base.uuid
    }

    fn already_refund(&self) -> bool {
        self.base.payment_info.refund_transaction_id.is_some()
    }

    fn order_status(&self) -> OrderStatus {
        self.base.order_status
    }

    fn order_time_info(&self) -> OrderTimeInfo {
        self.base.order_time_info
    }

    fn unit_price(&self) -> Decimal {
        self.base.unit_price
    }

    fn amount(&self) -> Decimal {
        self.base.amount
    }

    fn payment_info(&self) -> PaymentInfo {
        self.base.payment_info
    }
    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DishOrder {
    base: BaseOrder,
    train_order_id: OrderId,
    dish_id: DishId,
    unit_price: Decimal,
    amount: Decimal,
}

impl DishOrder {
    pub fn new(
        base_order: BaseOrder,
        train_order_id: OrderId,
        dish_id: DishId,
        unit_price: Decimal,
        amount: Decimal,
    ) -> Self {
        Self {
            base: base_order,
            train_order_id,
            dish_id,
            unit_price,
            amount,
        }
    }

    pub fn train_order_id(&self) -> OrderId {
        self.train_order_id
    }

    pub fn dish_id(&self) -> DishId {
        self.dish_id
    }

    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }
}

impl Identifiable for DishOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        Some(self.base.order_id)
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = id
    }
}

impl Entity for DishOrder {}
impl Aggregate for DishOrder {}

impl Order for DishOrder {
    fn order_id(&self) -> OrderId {
        self.base.order_id
    }

    fn uuid(&self) -> Uuid {
        self.base.uuid
    }

    fn already_refund(&self) -> bool {
        self.base.payment_info.refund_transaction_id.is_some()
    }

    fn order_status(&self) -> OrderStatus {
        self.base.order_status
    }

    fn order_time_info(&self) -> OrderTimeInfo {
        self.base.order_time_info
    }

    fn unit_price(&self) -> Decimal {
        self.base.unit_price
    }

    fn amount(&self) -> Decimal {
        self.base.amount
    }

    fn payment_info(&self) -> PaymentInfo {
        self.base.payment_info
    }
    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TakeawayOrder {
    base: BaseOrder,
    train_order_id: OrderId,
    takeaway_dish_id: TakeawayDishId,
    unit_price: Decimal,
    amount: Decimal,
}

impl TakeawayOrder {
    pub fn new(
        base_order: BaseOrder,
        train_order_id: OrderId,
        takeaway_dish_id: TakeawayDishId,
        unit_price: Decimal,
        amount: Decimal,
    ) -> Self {
        Self {
            base: base_order,
            train_order_id,
            takeaway_dish_id,
            unit_price,
            amount,
        }
    }

    pub fn train_order_id(&self) -> OrderId {
        self.train_order_id
    }

    pub fn takeaway_dish_id(&self) -> TakeawayDishId {
        self.takeaway_dish_id
    }

    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }
}

impl Order for TakeawayOrder {
    fn order_id(&self) -> OrderId {
        self.base.order_id
    }

    fn uuid(&self) -> Uuid {
        self.base.uuid
    }

    fn already_refund(&self) -> bool {
        self.base.payment_info.refund_transaction_id.is_some()
    }

    fn order_status(&self) -> OrderStatus {
        self.base.order_status
    }

    fn order_time_info(&self) -> OrderTimeInfo {
        self.base.order_time_info
    }

    fn unit_price(&self) -> Decimal {
        self.base.unit_price
    }

    fn amount(&self) -> Decimal {
        self.base.amount
    }

    fn payment_info(&self) -> PaymentInfo {
        self.base.payment_info
    }
    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }
}

impl Identifiable for TakeawayOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        Some(self.base.order_id)
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = id
    }
}

impl Entity for TakeawayOrder {}
impl Aggregate for TakeawayOrder {}
