//! # 订单实体模块
//!
//! 该模块定义了火车票订购系统中的订单相关实体数据结构及其相关操作。主要包含以下内容：
//!
//! - `OrderStatus`: 枚举类型，表示订单的状态。
//! - `OrderTimeInfo`: 结构体，表示订单的时间信息。
//! - `PaymentInfo`: 结构体，表示订单的支付信息。
//! - `Order`: 特性，定义了订单的基本操作。
//! - `BaseOrder`: 结构体，表示基础订单信息。
//! - `TrainOrder`: 结构体，表示火车票订单。
//! - `HotelOrder`: 结构体，表示酒店预订订单。
//! - `DishOrder`: 结构体，表示火车餐订单。
//! - `TakeawayOrder`: 结构体，表示外卖订单。
use super::personal_info::PreferredSeatLocation;
use crate::Verified;
use crate::domain::model::dish::DishId;
use crate::domain::model::hotel::{HotelDateRange, HotelId, HotelRoomTypeId};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::takeaway::TakeawayDishId;
use crate::domain::model::train::SeatTypeName;
use crate::domain::model::train_schedule::{Seat, StationRange, TrainScheduleId};
use crate::domain::model::transaction::TransactionId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use dyn_clone::{DynClone, clone_trait_object};
use id_macro::define_id_type;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use uuid::Uuid;

/// 枚举类型，表示订单的状态。
///
/// 主要包含以下状态：
/// - `Unpaid`: 订单已经生成，但尚未支付。
/// - `Paid`: 订单已经生成，并且支付成功，等待后端处理。
/// - `Ongoing`: 订单已被后端处理，并且成功，但尚未出行。
/// - `Active`: 订单正在出行中。
/// - `Completed`: 订单已完成。
/// - `Failed`: 订单处理失败。
/// - `Cancelled`: 订单已被用户取消。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    /// 将字符串尝试转换为 `OrderStatus` 枚举类型。
    ///
    /// Arguments:
    /// - `value`: 要转换的字符串。
    ///
    /// Returns:
    /// - 成功时返回 `OrderStatus` 枚举类型。
    /// - 失败时返回错误字符串。
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "unpaid" => OrderStatus::Unpaid,
            "paid" => OrderStatus::Paid,
            "ongoing" => OrderStatus::Ongoing,
            "active" => OrderStatus::Active,
            "completed" => OrderStatus::Completed,
            "failed" => OrderStatus::Failed,
            "cancelled" => OrderStatus::Cancelled,
            x => return Err(format!("Invalid order status: {}", x)),
        })
    }
}

impl From<OrderStatus> for &'static str {
    /// 将 `OrderStatus` 枚举类型转换为字符串。
    ///
    /// Returns:
    /// - 对应的字符串。
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
    /// 将 `OrderStatus` 枚举类型的引用转换为字符串。
    ///
    /// Returns:
    /// - 对应的字符串。
    fn from(value: &OrderStatus) -> Self {
        (*value).into()
    }
}

impl std::fmt::Display for OrderStatus {
    /// 格式化输出 `OrderStatus` 枚举类型。
    ///
    /// Arguments:
    /// - `f`: 格式化器。
    ///
    /// Returns:
    /// - 格式化结果。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&OrderStatus as Into<&'static str>>::into(self))
    }
}

define_id_type!(Order);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderTimeInfo {
    /// 结构体，表示订单的时间信息。
    ///
    /// 包含以下字段：
    /// - `create_time`: 订单创建时间。
    /// - `active_time`: 订单活动时间（即，变为“行程中”状态的时间，此时间后不再能退款）。
    /// - `complete_time`: 订单完成时间。
    create_time: DateTimeWithTimeZone,
    active_time: DateTimeWithTimeZone,
    complete_time: DateTimeWithTimeZone,
}

impl OrderTimeInfo {
    /// 创建一个新的 `OrderTimeInfo` 实例。
    ///
    /// Arguments:
    /// - `create_time`: 订单创建时间。
    /// - `active_time`: 订单活动时间。
    /// - `complete_time`: 订单完成时间。
    ///
    /// Returns:
    /// - 新创建的 `OrderTimeInfo` 实例。
    pub fn new(
        create_time: DateTimeWithTimeZone,
        active_time: DateTimeWithTimeZone,
        complete_time: DateTimeWithTimeZone,
    ) -> Self {
        Self {
            create_time,
            active_time,
            complete_time,
        }
    }

    /// 获取订单创建时间。
    ///
    /// Returns:
    /// - 订单创建时间。
    pub fn create_time(&self) -> DateTimeWithTimeZone {
        self.create_time
    }

    /// 获取订单活动时间。
    ///
    /// Returns:
    /// - 订单活动时间。
    pub fn active_time(&self) -> DateTimeWithTimeZone {
        self.active_time
    }

    /// 获取订单完成时间。
    ///
    /// Returns:
    /// - 订单完成时间。
    pub fn complete_time(&self) -> DateTimeWithTimeZone {
        self.complete_time
    }
}

/// 结构体，表示订单的支付信息。
///
/// 包含以下字段：
/// - `pay_transaction_id`: 支付交易的唯一标识符，可能为空。
/// - `refund_transaction_id`: 退款交易的唯一标识符，可能为空。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaymentInfo {
    pay_transaction_id: Option<TransactionId>,
    refund_transaction_id: Option<TransactionId>,
}

impl PaymentInfo {
    /// 创建一个新的 `PaymentInfo` 实例。
    ///
    /// Arguments:
    /// - `pay_transaction_id`: 支付交易的唯一标识符，可能为空。
    /// - `refund_transaction_id`: 退款交易的唯一标识符，可能为空。
    ///
    /// Returns:
    /// - 新创建的 `PaymentInfo` 实例。
    pub fn new(
        pay_transaction_id: Option<TransactionId>,
        refund_transaction_id: Option<TransactionId>,
    ) -> Self {
        Self {
            pay_transaction_id,
            refund_transaction_id,
        }
    }

    /// 获取支付交易的唯一标识符。
    ///
    /// Returns:
    /// - 支付交易的唯一标识符，可能为空。
    pub fn pay_transaction_id(&self) -> Option<TransactionId> {
        self.pay_transaction_id
    }

    /// 获取退款交易的唯一标识符。
    ///
    /// Returns:
    /// - 退款交易的唯一标识符，可能为空。
    pub fn refund_transaction_id(&self) -> Option<TransactionId> {
        self.refund_transaction_id
    }

    pub fn set_pay_transaction_id(&mut self, tx_id: TransactionId) {
        self.pay_transaction_id = Some(tx_id);
    }

    pub fn set_refund_transaction_id(&mut self, tx_id: TransactionId) {
        self.refund_transaction_id = Some(tx_id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    Train,
    Hotel,
    Dish,
    Takeaway,
}

impl OrderType {
    pub fn message_queue_name(&self) -> &'static str {
        match self {
            OrderType::Train => "order.train",
            OrderType::Hotel => "order.hotel",
            OrderType::Dish => "order.dish",
            OrderType::Takeaway => "order.takeaway",
        }
    }
}
impl From<&OrderType> for &'static str {
    fn from(value: &OrderType) -> Self {
        match value {
            OrderType::Train => "train",
            OrderType::Hotel => "hotel",
            OrderType::Dish => "dish",
            OrderType::Takeaway => "takeaway",
        }
    }
}

impl Display for OrderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&OrderType as Into<&'static str>>::into(self))
    }
}

/// 特性，定义了订单的基本操作。
///
/// 包含以下方法：
/// - `order_id`: 获取订单的唯一标识符。
/// - `uuid`: 获取订单的 UUID。
/// - `already_refund`: 判断订单是否已退款。
/// - `order_status`: 获取订单的状态。
/// - `order_time_info`: 获取订单的时间信息。
/// - `unit_price`: 获取订单的单价。
/// - `amount`: 获取订单的数量。
/// - `payment_info`: 获取订单的支付信息。
/// - `personal_info_id`: 获取订单关联的个人信息唯一标识符。
pub trait Order: DynClone + Debug + Send + Sync + 'static + Any {
    /// 获取订单的唯一标识符。
    ///
    /// Returns:
    /// - 订单的唯一标识符。
    fn order_id(&self) -> Option<OrderId>;

    /// 获取订单的 UUID。
    ///
    /// Returns:
    /// - 订单的 UUID。
    fn uuid(&self) -> Uuid;

    /// 判断订单是否已退款。
    ///
    /// Returns:
    /// - 如果订单已退款，返回 `true`；否则返回 `false`。
    fn already_refund(&self) -> bool;

    /// 获取订单的状态。
    ///
    /// Returns:
    /// - 订单的状态。
    fn order_status(&self) -> OrderStatus;

    fn order_type(&self) -> OrderType;

    /// 获取订单的时间信息。
    ///
    /// Returns:
    /// - 订单的时间信息。
    fn order_time_info(&self) -> OrderTimeInfo;

    /// 获取订单的单价。
    ///
    /// Returns:
    /// - 订单的单价。
    fn unit_price(&self) -> Decimal;

    /// 获取订单的数量。
    ///
    /// Returns:
    /// - 订单的数量。
    fn amount(&self) -> Decimal;

    /// 获取订单的支付信息。
    ///
    /// Returns:
    /// - 订单的支付信息。
    fn payment_info(&self) -> PaymentInfo;

    fn payment_info_mut(&mut self) -> &mut PaymentInfo;

    /// 获取订单关联的个人信息唯一标识符。
    ///
    /// Returns:
    /// - 订单关联的个人信息唯一标识符。
    fn personal_info_id(&self) -> PersonalInfoId;

    fn set_status(&mut self, status: OrderStatus);
}

clone_trait_object!(Order);

/// 结构体，表示基础订单信息。
///
/// 包含以下字段：
/// - `order_id`: 订单的唯一标识符。
/// - `uuid`: 订单的 UUID。
/// - `order_status`: 订单的状态。
/// - `order_time_info`: 订单的时间信息。
/// - `unit_price`: 订单的单价。
/// - `amount`: 订单的数量。
/// - `payment_info`: 订单的支付信息。
/// - `personal_info_id`: 订单关联的个人信息唯一标识符。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BaseOrder {
    pub order_id: Option<OrderId>,
    pub uuid: Uuid,
    pub order_status: OrderStatus,
    pub order_time_info: OrderTimeInfo,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub payment_info: PaymentInfo,
    pub personal_info_id: PersonalInfoId,
}

impl BaseOrder {
    /// 创建一个新的 `BaseOrder` 实例。
    ///
    /// Arguments:
    /// - `order_id`: 订单的唯一标识符。
    /// - `uuid`: 订单的 UUID。
    /// - `order_status`: 订单的状态。
    /// - `order_time_info`: 订单的时间信息。
    /// - `unit_price`: 订单的单价。
    /// - `amount`: 订单的数量。
    /// - `payment_info`: 订单的支付信息。
    /// - `personal_info_id`: 订单关联的个人信息唯一标识符。
    ///
    /// Returns:
    /// - 新创建的 `BaseOrder` 实例。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        order_id: Option<OrderId>,
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

/// 结构体，表示火车票订单。
///
/// 包含以下字段：
/// - `base`: 基础订单信息。
/// - `train_schedule_id`: 火车时刻表的唯一标识符。
/// - `seat`: 座位信息。
/// - `station_range`: 站点范围。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrainOrder {
    base: BaseOrder,
    train_schedule_id: TrainScheduleId,
    seat: Option<Seat>,
    order_seat_type_name: SeatTypeName<Verified>,
    preferred_seat_location: Option<PreferredSeatLocation>,
    station_range: StationRange<Verified>,
}

impl TrainOrder {
    /// 创建一个新的 `TrainOrder` 实例。
    ///
    /// Arguments:
    /// - `base_order`: 基础订单信息。
    /// - `train_schedule_id`: 火车时刻表的唯一标识符。
    /// - `seat`: 座位信息。
    /// - `station_range`: 站点范围。
    ///
    /// Returns:
    /// - 新创建的 `TrainOrder` 实例。
    pub fn new(
        base_order: BaseOrder,
        train_schedule_id: TrainScheduleId,
        seat: Option<Seat>,
        order_seat_type_name: SeatTypeName<Verified>,
        preferred_seat_location: Option<PreferredSeatLocation>,
        station_range: StationRange<Verified>,
    ) -> Self {
        Self {
            base: base_order,
            train_schedule_id,
            seat,
            order_seat_type_name,
            preferred_seat_location,
            station_range,
        }
    }

    /// 获取火车时刻表的唯一标识符。
    ///
    /// Returns:
    /// - 火车时刻表的唯一标识符。
    pub fn train_schedule_id(&self) -> TrainScheduleId {
        self.train_schedule_id
    }

    /// 获取站点范围。
    ///
    /// Returns:
    /// - 站点范围。
    pub fn station_range(&self) -> StationRange<Verified> {
        self.station_range
    }

    /// 获取座位信息。
    ///
    /// Returns:
    /// - 座位信息的引用。
    pub fn seat(&self) -> &Option<Seat> {
        &self.seat
    }

    /// 获取首选座位位置。
    ///
    /// Returns:
    /// - 首选座位位置的引用。
    pub fn preferred_seat_location(&self) -> &Option<PreferredSeatLocation> {
        &self.preferred_seat_location
    }

    /// 设置座位信息
    ///
    /// Arguments:
    /// - `seat`: 新的座位信息
    pub fn set_seat(&mut self, seat: Option<Seat>) {
        self.seat = seat;
    }

    /// 设置首选座位位置
    ///
    /// Arguments:
    /// - `location`: 新的首选座位位置
    pub fn set_preferred_seat_location(&mut self, location: Option<PreferredSeatLocation>) {
        self.preferred_seat_location = location;
    }

    pub fn base(&self) -> &BaseOrder {
        &self.base
    }

    pub fn order_seat_type_name(&self) -> &SeatTypeName<Verified> {
        &self.order_seat_type_name
    }
}

impl Order for TrainOrder {
    fn order_id(&self) -> Option<OrderId> {
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

    fn order_type(&self) -> OrderType {
        OrderType::Train
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

    fn payment_info_mut(&mut self) -> &mut PaymentInfo {
        &mut self.base.payment_info
    }

    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }

    fn set_status(&mut self, status: OrderStatus) {
        self.base.order_status = status;
    }
}

impl Identifiable for TrainOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        self.base.order_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = Some(id)
    }
}

impl Entity for TrainOrder {}
impl Aggregate for TrainOrder {}

/// 结构体，表示酒店预订订单。
///
/// 包含以下字段：
/// - `base`: 基础订单信息。
/// - `hotel_id`: 酒店的唯一标识符。
/// - `room_id`: 房间的唯一标识符。
/// - `booking_date_range`: 预订日期范围。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotelOrder {
    base: BaseOrder,
    hotel_id: HotelId,
    room_id: HotelRoomTypeId,
    booking_date_range: HotelDateRange,
}

impl HotelOrder {
    /// 创建一个新的 `HotelOrder` 实例。
    ///
    /// Arguments:
    /// - `base_order`: 基础订单信息。
    /// - `hotel_id`: 酒店的唯一标识符。
    /// - `room_id`: 房间的唯一标识符。
    /// - `booking_date_range`: 预订日期范围。
    ///
    /// Returns:
    /// - 新创建的 `HotelOrder` 实例。
    pub fn new(
        base_order: BaseOrder,
        hotel_id: HotelId,
        room_id: HotelRoomTypeId,
        booking_date_range: HotelDateRange,
    ) -> Self {
        Self {
            base: base_order,
            hotel_id,
            room_id,
            booking_date_range,
        }
    }

    /// 获取酒店的唯一标识符。
    ///
    /// Returns:
    /// - 酒店的唯一标识符。
    pub fn hotel_id(&self) -> HotelId {
        self.hotel_id
    }

    /// 获取房间的唯一标识符。
    ///
    /// Returns:
    /// - 房间的唯一标识符。
    pub fn room_id(&self) -> HotelRoomTypeId {
        self.room_id
    }

    /// 获取预订日期范围。
    ///
    /// Returns:
    /// - 预订日期范围。
    pub fn booking_date_range(&self) -> HotelDateRange {
        self.booking_date_range
    }

    pub fn base(&self) -> &BaseOrder {
        &self.base
    }
}

impl Identifiable for HotelOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        self.base.order_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = Some(id)
    }
}

impl Entity for HotelOrder {}
impl Aggregate for HotelOrder {}

impl Order for HotelOrder {
    fn order_id(&self) -> Option<OrderId> {
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

    fn order_type(&self) -> OrderType {
        OrderType::Hotel
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

    fn payment_info_mut(&mut self) -> &mut PaymentInfo {
        &mut self.base.payment_info
    }

    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }

    fn set_status(&mut self, status: OrderStatus) {
        self.base.order_status = status;
    }
}

/// 结构体，表示火车餐订单。
///
/// 包含以下字段：
/// - `base`: 基础订单信息。
/// - `train_order_id`: 火车票订单的唯一标识符。
/// - `dish_id`: 餐点的唯一标识符。
/// - `unit_price`: 餐点的单价。
/// - `amount`: 餐点的数量。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DishOrder {
    base: BaseOrder,
    train_order_id: OrderId,
    dish_id: DishId,
    unit_price: Decimal,
    amount: Decimal,
}

impl DishOrder {
    /// 创建一个新的 `DishOrder` 实例。
    ///
    /// Arguments:
    /// - `base_order`: 基础订单信息。
    /// - `train_order_id`: 火车票订单的唯一标识符。
    /// - `dish_id`: 餐点的唯一标识符。
    /// - `unit_price`: 餐点的单价。
    /// - `amount`: 餐点的数量。
    ///
    /// Returns:
    /// - 新创建的 `DishOrder` 实例。
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

    /// 获取火车票订单的唯一标识符。
    ///
    /// Returns:
    /// - 火车票订单的唯一标识符。
    pub fn train_order_id(&self) -> OrderId {
        self.train_order_id
    }

    /// 获取餐点的唯一标识符。
    ///
    /// Returns:
    /// - 餐点的唯一标识符。
    pub fn dish_id(&self) -> DishId {
        self.dish_id
    }

    /// 获取餐点的单价。
    ///
    /// Returns:
    /// - 餐点的单价。
    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    /// 获取餐点的数量。
    ///
    /// Returns:
    /// - 餐点的数量。
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn base(&self) -> &BaseOrder {
        &self.base
    }
}

impl Identifiable for DishOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        self.base.order_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = Some(id);
    }
}

impl Entity for DishOrder {}
impl Aggregate for DishOrder {}

impl Order for DishOrder {
    fn order_id(&self) -> Option<OrderId> {
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

    fn order_type(&self) -> OrderType {
        OrderType::Dish
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

    fn payment_info_mut(&mut self) -> &mut PaymentInfo {
        &mut self.base.payment_info
    }

    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }

    fn set_status(&mut self, status: OrderStatus) {
        self.base.order_status = status;
    }
}

/// 结构体，表示外卖订单。
///
/// 包含以下字段：
/// - `base`: 基础订单信息。
/// - `train_order_id`: 火车票订单的唯一标识符。
/// - `takeaway_dish_id`: 外卖的唯一标识符。
/// - `unit_price`: 外卖的单价。
/// - `amount`: 外卖的数量。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TakeawayOrder {
    base: BaseOrder,
    train_order_id: OrderId,
    takeaway_dish_id: TakeawayDishId,
    unit_price: Decimal,
    amount: Decimal,
}

impl TakeawayOrder {
    /// 创建一个新的 `TakeawayOrder` 实例。
    ///
    /// Arguments:
    /// - `base_order`: 基础订单信息。
    /// - `train_order_id`: 火车票订单的唯一标识符。
    /// - `takeaway_dish_id`: 外卖的唯一标识符。
    /// - `unit_price`: 外卖的单价。
    /// - `amount`: 外卖的数量。
    ///
    /// Returns:
    /// - 新创建的 `TakeawayOrder` 实例。
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

    /// 获取火车票订单的唯一标识符。
    ///
    /// Returns:
    /// - 火车票订单的唯一标识符。
    pub fn train_order_id(&self) -> OrderId {
        self.train_order_id
    }

    /// 获取外卖的唯一标识符。
    ///
    /// Returns:
    /// - 外卖的唯一标识符。
    pub fn takeaway_dish_id(&self) -> TakeawayDishId {
        self.takeaway_dish_id
    }

    /// 获取外卖的单价。
    ///
    /// Returns:
    /// - 外卖的单价。
    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    /// 获取外卖的数量。
    ///
    /// Returns:
    /// - 外卖的数量。
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn base(&self) -> &BaseOrder {
        &self.base
    }
}

impl Order for TakeawayOrder {
    fn order_id(&self) -> Option<OrderId> {
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

    fn order_type(&self) -> OrderType {
        OrderType::Takeaway
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

    fn payment_info_mut(&mut self) -> &mut PaymentInfo {
        &mut self.base.payment_info
    }

    fn personal_info_id(&self) -> PersonalInfoId {
        self.base.personal_info_id
    }

    fn set_status(&mut self, status: OrderStatus) {
        self.base.order_status = status;
    }
}

impl Identifiable for TakeawayOrder {
    type ID = OrderId;

    fn get_id(&self) -> Option<Self::ID> {
        self.base.order_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.base.order_id = Some(id)
    }
}

impl Entity for TakeawayOrder {}
impl Aggregate for TakeawayOrder {}
