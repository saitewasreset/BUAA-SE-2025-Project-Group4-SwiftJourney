//! 交易仓储实现模块
//!
//! 本模块提供了交易(Transaction)和订单(Order)的数据库仓储实现，负责处理交易数据的持久化操作。
//!
//! ## 主要功能
//! - 交易记录的CRUD操作
//! - 关联订单(火车票、酒店、餐饮、外卖)的批量处理
//! - 用户余额查询
//! - 数据转换(领域模型 ↔ 数据库模型)
//!
//! ## 模块结构
//! - `TransactionRepositoryImpl`: 交易仓储主实现
//! - `OrderDataConverter`: 订单数据转换器
//! - `TransactionDataConverter`: 交易数据转换器
//! - 各种数据包结构体: 用于批量处理相关数据
//!
//! ## 数据库表关系
//! - 交易表(transaction)为主表
//! - 订单表(train_order/hotel_order/dish_order/takeaway_order)为从表
//! - 通过外键关联(pay_transaction_id/refund_transaction_id)
//!
//! ## 注意事项
//! - 所有操作都在数据库事务中执行以保证数据一致性
//! - 使用聚合管理器跟踪变更
//! - 支持四种订单类型的混合处理
use crate::domain::model::dish::DishId;
use crate::domain::model::hotel::{HotelDateRange, HotelId, HotelRoomId};
use crate::domain::model::order::{
    BaseOrder, DishOrder, HotelOrder, Order, OrderId, OrderStatus, OrderTimeInfo, PaymentInfo,
    TakeawayOrder, TrainOrder,
};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::station::StationId;
use crate::domain::model::takeaway::TakeawayDishId;
use crate::domain::model::train::{SeatType, SeatTypeId, SeatTypeName};
use crate::domain::model::train_schedule::{
    Seat, SeatId, SeatLocationInfo, SeatStatus, StationRange, TrainScheduleId,
};
use crate::domain::model::transaction::{Transaction, TransactionId, TransactionStatus};
use crate::domain::model::user::UserId;
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::service::{AggregateManagerImpl, DiffInfo};
use crate::domain::{DbId, DiffType, Identifiable, TypedDiff};
use crate::domain::{DbRepositorySupport, MultiEntityDiff, RepositoryError};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::{One, ToPrimitive};
use sea_orm::{
    ActiveValue, DatabaseBackend, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, Select, Statement, TransactionTrait,
};
use sea_orm::{ColumnTrait, FromQueryResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

impl_db_id_from_u64!(OrderId, i32, "order id");

/// 订单数据转换器
///
/// 提供订单模型在领域对象和数据库对象之间的转换功能
/// 支持四种订单类型: 火车票、酒店、火车餐、外卖
pub struct OrderDataConverter;

/// 火车票订单数据包
///
/// 包含火车票订单模型及其关联的座位类型和座位映射数据
pub struct TrainOrderDoPack {
    train_order: crate::models::train_order::Model,
    /// 座位类型字典(seat_type_id → 模型)
    seat_type: HashMap<i32, crate::models::seat_type::Model>,
    /// 座位位置映射字典(seat_type_id → seat_id → 映射模型)
    seat_type_mapping: HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
}

impl OrderDataConverter {
    /// 从数据库模型创建火车票订单领域对象
    ///
    /// # Arguments
    /// * `train_order_do_pack` - 包含火车票订单及相关数据的包
    ///
    /// # Returns
    /// 返回转换后的`TrainOrder`结果
    ///
    /// # Errors
    /// 当数据不一致或转换失败时返回错误
    pub fn make_from_do_train(
        train_order_do_pack: TrainOrderDoPack,
    ) -> Result<TrainOrder, anyhow::Error> {
        let train_order_do = train_order_do_pack.train_order;

        let order_id = OrderId::from_db_value(train_order_do.id)?;
        let uuid = train_order_do.uuid;
        let order_status =
            OrderStatus::try_from(train_order_do.status.as_str()).map_err(|e| anyhow!(e))?;
        let order_time_info = OrderTimeInfo::new(
            train_order_do.create_time,
            train_order_do.active_time,
            train_order_do.complete_time,
        );
        let unit_price = train_order_do.price;
        let amount = Decimal::one();
        let payment_info = PaymentInfo::new(
            train_order_do
                .pay_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
            train_order_do
                .refund_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
        );
        let personal_info_id = PersonalInfoId::from_db_value(train_order_do.person_info_id)?;

        let base = BaseOrder::new(
            order_id,
            uuid,
            order_status,
            order_time_info,
            unit_price,
            amount,
            payment_info,
            personal_info_id,
        );

        let train_schedule_id = TrainScheduleId::from_db_value(train_order_do.train_schedule_id)?;

        let station_range = StationRange::from_unchecked(
            StationId::try_from(train_order_do.begin_station_id)?,
            StationId::try_from(train_order_do.end_station_id)?,
        );

        let seat_type = train_order_do_pack.seat_type;
        let seat_type_mapping = train_order_do_pack.seat_type_mapping;

        let seat_type_do = seat_type
            .get(&train_order_do.seat_type_id)
            .context(format!(
                "Inconsistent: cannot find seat type id: {}",
                train_order_do.seat_type_id
            ))?;

        let seat_type = SeatType::new(
            Some(SeatTypeId::try_from(seat_type_do.id)?),
            SeatTypeName::from_unchecked(seat_type_do.type_name.clone()),
            seat_type_do.capacity as u32,
            seat_type_do.price,
        );

        let seat_type_mapping_do = seat_type_mapping
            .get(&train_order_do.seat_type_id)
            .context(format!(
                "Inconsistent: cannot find seat type mapping with seat type id: {}",
                train_order_do.seat_type_id
            ))?
            .get(&(train_order_do.seat_id as i64))
            .context(format!(
                "Inconsistent: cannot find seat type mapping with seat id: {}",
                train_order_do.seat_id
            ))?;

        let seat_location_info = SeatLocationInfo {
            carriage: seat_type_mapping_do.carriage,
            row: seat_type_mapping_do.row,
            location: seat_type_mapping_do
                .location
                .chars()
                .next()
                .expect("location should not be empty"),
        };

        let seat = Seat::new(
            SeatId::from_db_value(train_order_do.seat_type_id as i64)?,
            seat_type,
            seat_location_info,
            SeatStatus::Occupied,
        );

        Ok(TrainOrder::new(
            base,
            train_schedule_id,
            seat,
            station_range,
        ))
    }

    /// 将火车票订单领域对象转换为数据库模型
    ///
    /// # Arguments
    /// * `train_order` - 要转换的火车票订单
    ///
    /// # Returns
    /// 返回转换后的数据库活动模型
    pub fn transform_to_do_train(
        train_order: TrainOrder,
    ) -> crate::models::train_order::ActiveModel {
        let mut model = crate::models::train_order::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(train_order.uuid()),
            status: ActiveValue::Set(
                <OrderStatus as Into<&str>>::into(train_order.order_status()).to_string(),
            ),
            create_time: ActiveValue::Set(train_order.order_time_info().crate_time()),
            active_time: ActiveValue::Set(train_order.order_time_info().active_time()),
            complete_time: ActiveValue::Set(train_order.order_time_info().complete_time()),
            price: ActiveValue::Set(train_order.unit_price()),
            pay_transaction_id: ActiveValue::NotSet,
            refund_transaction_id: ActiveValue::NotSet,
            person_info_id: ActiveValue::Set(train_order.personal_info_id().to_db_value()),

            train_schedule_id: ActiveValue::Set(train_order.train_schedule_id().to_db_value()),
            seat_type_id: ActiveValue::Set(
                train_order
                    .seat()
                    .seat_type()
                    .get_id()
                    .expect("seat type should have id")
                    .to_db_value(),
            ),
            seat_id: ActiveValue::Set(
                train_order
                    .seat()
                    .get_id()
                    .expect("seat should have id")
                    .to_db_value() as i32,
            ),
            begin_station_id: ActiveValue::Set(
                train_order
                    .station_range()
                    .get_from_station_id()
                    .to_db_value(),
            ),
            end_station_id: ActiveValue::Set(
                train_order
                    .station_range()
                    .get_to_station_id()
                    .to_db_value(),
            ),
        };

        if let Some(id) = train_order.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        if let Some(id) = train_order.payment_info().pay_transaction_id() {
            model.pay_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        if let Some(id) = train_order.payment_info().refund_transaction_id() {
            model.refund_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        model
    }

    /// 从数据库模型创建酒店订单领域对象
    ///
    /// # Arguments
    /// * `hotel_order_do` - 包含酒店订单及相关数据的包
    ///
    /// # Returns
    /// 返回转换后的`HotelOrder`结果
    ///
    /// # Errors
    /// 当数据不一致或转换失败时返回错误
    pub fn make_from_do_hotel(
        hotel_order_do: crate::models::hotel_order::Model,
    ) -> Result<HotelOrder, anyhow::Error> {
        let order_id = OrderId::from_db_value(hotel_order_do.id)?;
        let uuid = hotel_order_do.uuid;
        let order_status =
            OrderStatus::try_from(hotel_order_do.status.as_str()).map_err(|e| anyhow!(e))?;
        let order_time_info = OrderTimeInfo::new(
            hotel_order_do.create_time,
            hotel_order_do.active_time,
            hotel_order_do.complete_time,
        );
        let unit_price = hotel_order_do.price;
        let amount = Decimal::one();
        let payment_info = PaymentInfo::new(
            hotel_order_do
                .pay_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
            hotel_order_do
                .refund_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
        );
        let personal_info_id = PersonalInfoId::from_db_value(hotel_order_do.person_info_id as i32)?;

        let base = BaseOrder::new(
            order_id,
            uuid,
            order_status,
            order_time_info,
            unit_price,
            amount,
            payment_info,
            personal_info_id,
        );

        let hotel_id = HotelId::from_db_value(hotel_order_do.hotel_id as i32)?;

        let date_range = HotelDateRange::new(hotel_order_do.begin_date, hotel_order_do.end_date)?;

        let hotel_room_type_id =
            HotelRoomId::from_db_value(hotel_order_do.hotel_room_type_id as i32)?;

        Ok(HotelOrder::new(
            base,
            hotel_id,
            hotel_room_type_id,
            date_range,
        ))
    }

    /// 将酒店订单领域对象转换为数据库模型
    ///
    /// # Arguments
    /// * `hotel_order` - 要转换的酒店订单
    ///
    /// # Returns
    /// 返回转换后的数据库活动模型
    pub fn transform_to_do_hotel(
        hotel_order: HotelOrder,
    ) -> crate::models::hotel_order::ActiveModel {
        let mut model = crate::models::hotel_order::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(hotel_order.uuid()),
            status: ActiveValue::Set(
                <OrderStatus as Into<&str>>::into(hotel_order.order_status()).to_string(),
            ),
            create_time: ActiveValue::Set(hotel_order.order_time_info().crate_time()),
            active_time: ActiveValue::Set(hotel_order.order_time_info().active_time()),
            complete_time: ActiveValue::Set(hotel_order.order_time_info().complete_time()),
            price: ActiveValue::Set(hotel_order.unit_price()),
            pay_transaction_id: ActiveValue::NotSet,
            refund_transaction_id: ActiveValue::NotSet,
            person_info_id: ActiveValue::Set(hotel_order.personal_info_id().to_db_value() as i64),

            hotel_id: ActiveValue::Set(hotel_order.hotel_id().to_db_value() as i64),
            begin_date: ActiveValue::Set(hotel_order.booking_date_range().begin_date()),
            end_date: ActiveValue::Set(hotel_order.booking_date_range().end_date()),
            hotel_room_type_id: ActiveValue::Set(hotel_order.room_id().to_db_value() as i64),
        };

        if let Some(id) = hotel_order.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        if let Some(id) = hotel_order.payment_info().pay_transaction_id() {
            model.pay_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        if let Some(id) = hotel_order.payment_info().refund_transaction_id() {
            model.refund_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        model
    }

    /// 从数据库模型创建火车餐订单领域对象
    ///
    /// # Arguments
    /// * `dish_order_do` - 包含火车餐订单及相关数据的包
    ///
    /// # Returns
    /// 返回转换后的`DishOrder`结果
    ///
    /// # Errors
    /// 当数据不一致或转换失败时返回错误
    pub fn make_from_do_dish(
        dish_order_do: crate::models::dish_order::Model,
    ) -> Result<DishOrder, anyhow::Error> {
        let order_id = OrderId::from_db_value(dish_order_do.id)?;
        let uuid = dish_order_do.uuid;
        let order_status =
            OrderStatus::try_from(dish_order_do.status.as_str()).map_err(|e| anyhow!(e))?;
        let order_time_info = OrderTimeInfo::new(
            dish_order_do.create_time,
            dish_order_do.active_time,
            dish_order_do.complete_time,
        );
        let unit_price = dish_order_do.price;
        let amount = Decimal::from(dish_order_do.amount);
        let payment_info = PaymentInfo::new(
            dish_order_do
                .pay_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
            dish_order_do
                .refund_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
        );
        let personal_info_id = PersonalInfoId::from_db_value(dish_order_do.person_info_id as i32)?;

        let base = BaseOrder::new(
            order_id,
            uuid,
            order_status,
            order_time_info,
            unit_price,
            amount,
            payment_info,
            personal_info_id,
        );

        let train_order_id = OrderId::from_db_value(dish_order_do.train_order_id as i32)?;

        let dish_id = DishId::from_db_value(dish_order_do.dish_id as i32)?;

        Ok(DishOrder::new(
            base,
            train_order_id,
            dish_id,
            unit_price,
            amount,
        ))
    }

    /// 将火车餐订单领域对象转换为数据库模型
    ///
    /// # Arguments
    /// * `dish_order` - 要转换的火车餐订单
    ///
    /// # Returns
    /// 返回转换后的数据库活动模型
    pub fn transform_to_do_dish(dish_order: DishOrder) -> crate::models::dish_order::ActiveModel {
        let mut model = crate::models::dish_order::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(dish_order.uuid()),
            status: ActiveValue::Set(
                <OrderStatus as Into<&str>>::into(dish_order.order_status()).to_string(),
            ),
            create_time: ActiveValue::Set(dish_order.order_time_info().crate_time()),
            active_time: ActiveValue::Set(dish_order.order_time_info().active_time()),
            complete_time: ActiveValue::Set(dish_order.order_time_info().complete_time()),
            price: ActiveValue::Set(dish_order.unit_price()),
            pay_transaction_id: ActiveValue::NotSet,
            refund_transaction_id: ActiveValue::NotSet,
            person_info_id: ActiveValue::Set(dish_order.personal_info_id().to_db_value() as i64),

            train_order_id: ActiveValue::Set(dish_order.train_order_id().to_db_value() as i64),
            dish_id: ActiveValue::Set(dish_order.dish_id().to_db_value() as i64),
            amount: ActiveValue::Set(dish_order.amount().to_i32().unwrap()),
        };

        if let Some(id) = dish_order.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        if let Some(id) = dish_order.payment_info().pay_transaction_id() {
            model.pay_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        if let Some(id) = dish_order.payment_info().refund_transaction_id() {
            model.refund_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        model
    }

    /// 从数据库模型创建外卖订单领域对象
    ///
    /// # Arguments
    /// * `takeaway_order_do` - 包含外卖订单及相关数据的包
    ///
    /// # Returns
    /// 返回转换后的`TakeawayOrder`结果
    ///
    /// # Errors
    /// 当数据不一致或转换失败时返回错误
    pub fn make_from_do_takeaway(
        takeaway_order_do: crate::models::takeaway_order::Model,
    ) -> Result<TakeawayOrder, anyhow::Error> {
        let order_id = OrderId::from_db_value(takeaway_order_do.id)?;
        let uuid = takeaway_order_do.uuid;
        let order_status =
            OrderStatus::try_from(takeaway_order_do.status.as_str()).map_err(|e| anyhow!(e))?;
        let order_time_info = OrderTimeInfo::new(
            takeaway_order_do.create_time,
            takeaway_order_do.active_time,
            takeaway_order_do.complete_time,
        );
        let unit_price = takeaway_order_do.price;
        let amount = Decimal::from(takeaway_order_do.amount);
        let payment_info = PaymentInfo::new(
            takeaway_order_do
                .pay_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
            takeaway_order_do
                .refund_transaction_id
                .map(TransactionId::try_from)
                .transpose()?,
        );
        let personal_info_id =
            PersonalInfoId::from_db_value(takeaway_order_do.person_info_id as i32)?;

        let base = BaseOrder::new(
            order_id,
            uuid,
            order_status,
            order_time_info,
            unit_price,
            amount,
            payment_info,
            personal_info_id,
        );

        let train_order_id = OrderId::from_db_value(takeaway_order_do.train_order_id as i32)?;

        let takeaway_dish_id =
            TakeawayDishId::from_db_value(takeaway_order_do.takeaway_dish_id as i32)?;

        Ok(TakeawayOrder::new(
            base,
            train_order_id,
            takeaway_dish_id,
            unit_price,
            amount,
        ))
    }

    /// 将外卖订单领域对象转换为数据库模型
    ///
    /// # Arguments
    /// * `takeaway_order` - 要转换的外卖订单
    ///
    /// # Returns
    /// 返回转换后的数据库活动模型
    pub fn transform_to_do_takeaway(
        takeaway_order: TakeawayOrder,
    ) -> crate::models::takeaway_order::ActiveModel {
        let mut model = crate::models::takeaway_order::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(takeaway_order.uuid()),
            status: ActiveValue::Set(
                <OrderStatus as Into<&str>>::into(takeaway_order.order_status()).to_string(),
            ),
            create_time: ActiveValue::Set(takeaway_order.order_time_info().crate_time()),
            active_time: ActiveValue::Set(takeaway_order.order_time_info().active_time()),
            complete_time: ActiveValue::Set(takeaway_order.order_time_info().complete_time()),
            price: ActiveValue::Set(takeaway_order.unit_price()),
            pay_transaction_id: ActiveValue::NotSet,
            refund_transaction_id: ActiveValue::NotSet,
            person_info_id: ActiveValue::Set(takeaway_order.personal_info_id().to_db_value() as i64),

            train_order_id: ActiveValue::Set(takeaway_order.train_order_id().to_db_value() as i64),
            takeaway_dish_id: ActiveValue::Set(
                takeaway_order.takeaway_dish_id().to_db_value() as i64
            ),
            amount: ActiveValue::Set(takeaway_order.amount().to_i32().unwrap()),
        };

        if let Some(id) = takeaway_order.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        if let Some(id) = takeaway_order.payment_info().pay_transaction_id() {
            model.pay_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        if let Some(id) = takeaway_order.payment_info().refund_transaction_id() {
            model.refund_transaction_id = ActiveValue::Set(Some(id.to_db_value() as i64));
        }

        model
    }
}

/// 交易数据转换器
///
/// 提供交易模型在领域对象和数据库对象之间的转换功能
pub struct TransactionDataConverter;

/// 订单数据包
///
/// 包含四种类型的订单列表，用于批量处理
pub struct OrderPack {
    pub train_orders: Vec<TrainOrder>,
    pub hotel_orders: Vec<HotelOrder>,
    pub dish_orders: Vec<DishOrder>,
    pub takeaway_orders: Vec<TakeawayOrder>,
}

impl From<Vec<Box<dyn Order>>> for OrderPack {
    fn from(orders: Vec<Box<dyn Order>>) -> Self {
        let mut train_orders = Vec::new();
        let mut hotel_orders = Vec::new();
        let mut dish_orders = Vec::new();
        let mut takeaway_orders = Vec::new();

        for order in orders {
            let order = order as Box<dyn Any>;
            match order.as_ref().type_id() {
                id if id == TypeId::of::<TrainOrder>() => {
                    train_orders.push(*order.downcast::<TrainOrder>().unwrap());
                }
                id if id == TypeId::of::<HotelOrder>() => {
                    hotel_orders.push(*order.downcast::<HotelOrder>().unwrap());
                }
                id if id == TypeId::of::<DishOrder>() => {
                    dish_orders.push(*order.downcast::<DishOrder>().unwrap());
                }
                id if id == TypeId::of::<TakeawayOrder>() => {
                    takeaway_orders.push(*order.downcast::<TakeawayOrder>().unwrap());
                }
                _ => panic!("Unknown order type"),
            }
        }

        OrderPack {
            train_orders,
            hotel_orders,
            dish_orders,
            takeaway_orders,
        }
    }
}

impl From<OrderPack> for Vec<Box<dyn Order>> {
    fn from(value: OrderPack) -> Self {
        let mut result: Vec<Box<dyn Order>> = Vec::new();

        for order in value.train_orders {
            result.push(Box::new(order));
        }

        for order in value.hotel_orders {
            result.push(Box::new(order));
        }

        for order in value.dish_orders {
            result.push(Box::new(order));
        }

        for order in value.takeaway_orders {
            result.push(Box::new(order));
        }

        result
    }
}

impl OrderPack {
    pub fn into_active_model(self) -> OrderActiveModelPack {
        let mut train_orders = Vec::new();
        let mut hotel_orders = Vec::new();
        let mut dish_orders = Vec::new();
        let mut takeaway_orders = Vec::new();

        for order in self.train_orders {
            train_orders.push(OrderDataConverter::transform_to_do_train(order));
        }

        for order in self.hotel_orders {
            hotel_orders.push(OrderDataConverter::transform_to_do_hotel(order));
        }

        for order in self.dish_orders {
            dish_orders.push(OrderDataConverter::transform_to_do_dish(order));
        }

        for order in self.takeaway_orders {
            takeaway_orders.push(OrderDataConverter::transform_to_do_takeaway(order));
        }

        OrderActiveModelPack {
            train_orders,
            hotel_orders,
            dish_orders,
            takeaway_orders,
        }
    }

    pub async fn delete_all(self, txn: &DatabaseTransaction) -> Result<(), DbErr> {
        for order in self.train_orders {
            if let Some(id) = order.get_id() {
                crate::models::train_order::Entity::delete_by_id(id.to_db_value())
                    .exec(txn)
                    .await?;
            }
        }

        for order in self.hotel_orders {
            if let Some(id) = order.get_id() {
                crate::models::hotel_order::Entity::delete_by_id(id.to_db_value())
                    .exec(txn)
                    .await?;
            }
        }

        for order in self.dish_orders {
            if let Some(id) = order.get_id() {
                crate::models::dish_order::Entity::delete_by_id(id.to_db_value())
                    .exec(txn)
                    .await?;
            }
        }

        for order in self.takeaway_orders {
            if let Some(id) = order.get_id() {
                crate::models::takeaway_order::Entity::delete_by_id(id.to_db_value())
                    .exec(txn)
                    .await?;
            }
        }

        Ok(())
    }
}

impl_db_id_from_u64!(TransactionId, i32, "transaction id");
/// 交易数据包
///
/// 包含交易模型及其关联的订单数据包
pub struct TransactionDoPack {
    pub transaction: crate::models::transaction::Model,
    pub orders: OrderPack,
}
/// 订单数据库活动模型包
///
/// 包含四种类型订单的数据库活动模型列表
pub struct OrderActiveModelPack {
    pub train_orders: Vec<crate::models::train_order::ActiveModel>,
    pub hotel_orders: Vec<crate::models::hotel_order::ActiveModel>,
    pub dish_orders: Vec<crate::models::dish_order::ActiveModel>,
    pub takeaway_orders: Vec<crate::models::takeaway_order::ActiveModel>,
}

impl OrderActiveModelPack {
    async fn insert_or_update_all(self, txn: &DatabaseTransaction) -> Result<(), DbErr> {
        let mut train_insert_list = Vec::new();
        let mut train_update_list = Vec::new();

        let mut hotel_insert_list = Vec::new();
        let mut hotel_update_list = Vec::new();

        let mut dish_insert_list = Vec::new();
        let mut dish_update_list = Vec::new();

        let mut takeaway_insert_list = Vec::new();
        let mut takeaway_update_list = Vec::new();

        for order in self.train_orders {
            if order.id.is_set() {
                train_update_list.push(order);
            } else {
                train_insert_list.push(order);
            }
        }

        for order in self.hotel_orders {
            if order.id.is_set() {
                hotel_update_list.push(order);
            } else {
                hotel_insert_list.push(order);
            }
        }

        for order in self.dish_orders {
            if order.id.is_set() {
                dish_update_list.push(order);
            } else {
                dish_insert_list.push(order);
            }
        }

        for order in self.takeaway_orders {
            if order.id.is_set() {
                takeaway_update_list.push(order);
            } else {
                takeaway_insert_list.push(order);
            }
        }

        crate::models::train_order::Entity::insert_many(train_insert_list)
            .exec(txn)
            .await?;

        for order in train_update_list {
            crate::models::train_order::Entity::update(order)
                .exec(txn)
                .await?;
        }

        crate::models::hotel_order::Entity::insert_many(hotel_insert_list)
            .exec(txn)
            .await?;

        for order in hotel_update_list {
            crate::models::hotel_order::Entity::update(order)
                .exec(txn)
                .await?;
        }

        crate::models::dish_order::Entity::insert_many(dish_insert_list)
            .exec(txn)
            .await?;
        for order in dish_update_list {
            crate::models::dish_order::Entity::update(order)
                .exec(txn)
                .await?;
        }

        crate::models::takeaway_order::Entity::insert_many(takeaway_insert_list)
            .exec(txn)
            .await?;
        for order in takeaway_update_list {
            crate::models::takeaway_order::Entity::update(order)
                .exec(txn)
                .await?;
        }

        Ok(())
    }
}

pub struct OrderDoPack {
    pub train_orders: Vec<crate::models::train_order::Model>,
    pub hotel_orders: Vec<crate::models::hotel_order::Model>,
    pub dish_orders: Vec<crate::models::dish_order::Model>,
    pub takeaway_orders: Vec<crate::models::takeaway_order::Model>,
}

impl OrderDoPack {
    pub async fn from_db(
        transaction_id: i32,
        db: &DatabaseConnection,
    ) -> Result<OrderDoPack, DbErr> {
        let train_orders = crate::models::train_order::Entity::find()
            .filter(crate::models::train_order::Column::PayTransactionId.eq(transaction_id))
            .all(db)
            .await?;

        let hotel_orders = crate::models::hotel_order::Entity::find()
            .filter(crate::models::hotel_order::Column::PayTransactionId.eq(transaction_id))
            .all(db)
            .await?;

        let dish_orders = crate::models::dish_order::Entity::find()
            .filter(crate::models::dish_order::Column::PayTransactionId.eq(transaction_id))
            .all(db)
            .await?;

        let takeaway_orders = crate::models::takeaway_order::Entity::find()
            .filter(crate::models::takeaway_order::Column::PayTransactionId.eq(transaction_id))
            .all(db)
            .await?;

        Ok(OrderDoPack {
            train_orders,
            hotel_orders,
            dish_orders,
            takeaway_orders,
        })
    }
    pub fn into_order_pack(
        self,
        train_schedule_id_to_type_id: &HashMap<i32, i32>,
        seat_type: &HashMap<i32, crate::models::seat_type::Model>,
        seat_type_mapping_all: &HashMap<
            i32,
            HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
        >,
    ) -> Result<OrderPack, anyhow::Error> {
        let mut train_orders = Vec::with_capacity(self.train_orders.len());
        let mut hotel_orders = Vec::with_capacity(self.hotel_orders.len());
        let mut dish_orders = Vec::with_capacity(self.dish_orders.len());
        let mut takeaway_orders = Vec::with_capacity(self.takeaway_orders.len());

        for order in self.train_orders {
            let train_id = train_schedule_id_to_type_id
                .get(&order.train_schedule_id)
                .ok_or(anyhow!(
                    "Inconsistent: cannot find train type id for train schedule id: {}",
                    order.train_schedule_id
                ))?;

            train_orders.push(OrderDataConverter::make_from_do_train(TrainOrderDoPack {
                train_order: order,
                seat_type: seat_type.clone(),
                seat_type_mapping: seat_type_mapping_all
                    .get(train_id)
                    .ok_or(anyhow!(
                        "Inconsistent: cannot find seat type mapping for train type id: {}",
                        train_id
                    ))?
                    .clone(),
            })?);
        }

        for order in self.hotel_orders {
            hotel_orders.push(OrderDataConverter::make_from_do_hotel(order)?);
        }

        for order in self.dish_orders {
            dish_orders.push(OrderDataConverter::make_from_do_dish(order)?);
        }

        for order in self.takeaway_orders {
            takeaway_orders.push(OrderDataConverter::make_from_do_takeaway(order)?);
        }

        Ok(OrderPack {
            train_orders,
            hotel_orders,
            dish_orders,
            takeaway_orders,
        })
    }
}

/// 交易数据库活动模型包
///
/// 包含交易及其订单的数据库活动模型
pub struct TransactionActiveModelPack {
    pub transaction: crate::models::transaction::ActiveModel,
    pub orders: OrderActiveModelPack,
}

impl TransactionDataConverter {
    /// 从数据库模型创建交易领域对象
    ///
    /// # Arguments
    /// * `transaction_do_pack` - 包含交易及相关订单数据的包
    ///
    /// # Returns
    /// 返回转换后的`Transaction`结果
    ///
    /// # Errors
    /// 当数据不一致或转换失败时返回错误
    pub fn make_from_do(
        transaction_do_pack: TransactionDoPack,
    ) -> Result<Transaction, anyhow::Error> {
        let transaction_id = TransactionId::from_db_value(transaction_do_pack.transaction.id)?;

        let transaction_status =
            TransactionStatus::try_from(transaction_do_pack.transaction.status.as_str())
                .map_err(|e| anyhow!(e))?;

        let orders: Vec<Box<dyn Order>> = transaction_do_pack.orders.into();

        Ok(Transaction::new_full(
            Some(transaction_id),
            transaction_do_pack.transaction.uuid,
            transaction_do_pack.transaction.create_time,
            transaction_do_pack.transaction.finish_time,
            transaction_do_pack.transaction.amount,
            transaction_status,
            UserId::try_from(transaction_do_pack.transaction.user_id)?,
            orders,
        ))
    }

    /// 将交易领域对象转换为数据库模型(仅交易部分)
    ///
    /// # Arguments
    /// * `transaction` - 要转换的交易对象
    ///
    /// # Returns
    /// 返回交易部分的数据库活动模型
    pub fn transform_to_do_transaction_only(
        transaction: &Transaction,
    ) -> crate::models::transaction::ActiveModel {
        let mut transaction_model = crate::models::transaction::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(transaction.uuid()),
            create_time: ActiveValue::Set(transaction.create_time()),
            finish_time: ActiveValue::Set(transaction.finish_time()),
            amount: ActiveValue::Set(transaction.raw_amount()),
            status: ActiveValue::Set(
                <TransactionStatus as Into<&str>>::into(transaction.status()).to_string(),
            ),
            user_id: ActiveValue::Set(transaction.user_id().to_db_value() as i64),
        };

        if let Some(id) = transaction.get_id() {
            transaction_model.id = ActiveValue::Set(id.to_db_value());
        }

        transaction_model
    }

    /// 将完整交易领域对象转换为数据库模型包
    ///
    /// # Arguments
    /// * `transaction` - 要转换的完整交易对象
    ///
    /// # Returns
    /// 返回包含交易和订单的数据库活动模型包
    pub fn transform_to_do(transaction: Transaction) -> TransactionActiveModelPack {
        let transaction_model =
            TransactionDataConverter::transform_to_do_transaction_only(&transaction);

        let orders = OrderPack::from(transaction.into_orders()).into_active_model();

        TransactionActiveModelPack {
            transaction: transaction_model,
            orders,
        }
    }
}

/// 交易仓储实现
///
/// 实现`TransactionRepository`和`DbRepositorySupport` trait
/// 使用SeaORM进行数据库操作，支持事务和聚合管理
pub struct TransactionRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<Transaction>>>,
}

fn is_order_equal(a: &dyn Order, b: &dyn Order) -> bool {
    if a.type_id() != b.type_id() {
        return false;
    }

    if (a.uuid() != b.uuid())
        || (a.order_status() != b.order_status())
        || (a.order_time_info() != b.order_time_info())
        || (a.unit_price() != b.unit_price())
        || (a.amount() != b.amount())
        || (a.payment_info() != b.payment_info())
        || (a.personal_info_id() != b.personal_info_id())
    {
        return false;
    }

    match a.type_id() {
        id if id == TypeId::of::<TrainOrder>() => {
            let a = (a as &dyn Any).downcast_ref::<TrainOrder>().unwrap();
            let b = (b as &dyn Any).downcast_ref::<TrainOrder>().unwrap();

            a == b
        }
        id if id == TypeId::of::<HotelOrder>() => {
            let a = (a as &dyn Any).downcast_ref::<HotelOrder>().unwrap();
            let b = (b as &dyn Any).downcast_ref::<HotelOrder>().unwrap();

            a == b
        }
        id if id == TypeId::of::<DishOrder>() => {
            let a = (a as &dyn Any).downcast_ref::<DishOrder>().unwrap();
            let b = (b as &dyn Any).downcast_ref::<DishOrder>().unwrap();

            a == b
        }
        id if id == TypeId::of::<TakeawayOrder>() => {
            let a = (a as &dyn Any).downcast_ref::<TakeawayOrder>().unwrap();
            let b = (b as &dyn Any).downcast_ref::<TakeawayOrder>().unwrap();

            a == b
        }
        _ => panic!("Unknown order type"),
    }
}

impl TransactionRepositoryImpl {
    /// 创建新的交易仓储实例
    ///
    /// # Arguments
    /// * `db` - 数据库连接
    ///
    /// # Returns
    /// 返回新创建的仓储实例
    pub fn new(db: DatabaseConnection) -> Self {
        let detect_change_fn = |diff: DiffInfo<Transaction>| {
            let mut result = MultiEntityDiff::new();

            let old = diff.old;
            let new = diff.new;

            let old_orders_map = old
                .iter()
                .flat_map(|item| item.orders())
                .map(|order| (order.order_id().to_db_value(), order.clone()))
                .collect::<HashMap<_, _>>();
            let new_orders_map = new
                .iter()
                .flat_map(|item| item.orders())
                .map(|order| (order.order_id().to_db_value(), order.clone()))
                .collect::<HashMap<_, _>>();

            for (order_id, old_order) in &old_orders_map {
                if let Some(new_order) = new_orders_map.get(order_id) {
                    if !is_order_equal(old_order.as_ref(), new_order.as_ref()) {
                        result.add_change(TypedDiff::new(
                            DiffType::Modified,
                            Some(old_order.clone()),
                            Some(new_order.clone()),
                        ));
                    }
                } else {
                    result.add_change(TypedDiff::new(
                        DiffType::Removed,
                        Some(old_order.clone()),
                        None,
                    ));
                }
            }

            for (new_order_id, new_order) in &new_orders_map {
                if !old_orders_map.contains_key(new_order_id) {
                    result.add_change(TypedDiff::new(
                        DiffType::Added,
                        None,
                        Some((*new_order).clone()),
                    ));
                }
            }

            result
        };

        Self {
            db,
            aggregate_manager: Arc::new(Mutex::new(AggregateManagerImpl::new(Box::new(
                detect_change_fn,
            )))),
        }
    }

    pub async fn query_transaction(
        &self,
        builder: impl FnOnce(
            Select<crate::models::transaction::Entity>,
        ) -> Select<crate::models::transaction::Entity>,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let transaction_dos = builder(crate::models::transaction::Entity::find())
            .all(&self.db)
            .await
            .context("Failed to query transaction")?;

        let seat_type = crate::models::seat_type::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to query seat type")?;

        let seat_type_mapping = crate::models::seat_type_mapping::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to query seat type mapping")?;

        let seat_type = seat_type
            .into_iter()
            .map(|item| (item.id, item))
            .collect::<HashMap<_, _>>();

        let mut seat_type_mapping_all: HashMap<
            i32,
            HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
        > = HashMap::new();

        let train_schedules = crate::models::train_schedule::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to query train schedule")?;

        let train_type_id_map = train_schedules
            .into_iter()
            .map(|item| (item.id, item.train_id))
            .collect::<HashMap<_, _>>();

        for model in seat_type_mapping {
            seat_type_mapping_all
                .entry(model.train_type_id)
                .or_default()
                .entry(model.seat_type_id)
                .or_default()
                .insert(model.seat_id, model);
        }

        let mut transactions = Vec::new();

        for transaction_do in transaction_dos {
            let order_do_pack = OrderDoPack::from_db(transaction_do.id, &self.db)
                .await
                .context("Failed to query order")?;

            let orders = order_do_pack
                .into_order_pack(&train_type_id_map, &seat_type, &seat_type_mapping_all)
                .map_err(RepositoryError::InconsistentState)?;

            let transaction_do_pack = TransactionDoPack {
                transaction: transaction_do,
                orders,
            };

            let transaction = TransactionDataConverter::make_from_do(transaction_do_pack)?;

            transactions.push(transaction);
        }

        Ok(transactions)
    }
}

#[async_trait]
impl DbRepositorySupport<Transaction> for TransactionRepositoryImpl {
    type Manager = AggregateManagerImpl<Transaction>;

    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

    async fn on_insert(&self, aggregate: Transaction) -> Result<TransactionId, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("Failed to start transaction")?;

        let model_pack = TransactionDataConverter::transform_to_do(aggregate);

        let result = crate::models::transaction::Entity::insert(model_pack.transaction)
            .exec(&txn)
            .await
            .context("Failed to insert transaction")?;

        model_pack
            .orders
            .insert_or_update_all(&txn)
            .await
            .context("Failed to insert orders")?;

        txn.commit().await.context("Failed to commit transaction")?;

        Ok(TransactionId::from_db_value(result.last_insert_id)?)
    }

    async fn on_select(&self, id: TransactionId) -> Result<Option<Transaction>, RepositoryError> {
        let result = self
            .query_transaction(|q| {
                q.filter(crate::models::transaction::Column::Id.eq(id.to_db_value()))
            })
            .await?;

        Ok(result.into_iter().next())
    }

    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("Failed to start transaction")?;

        let mut to_add_orders = Vec::new();
        let mut to_update_orders = Vec::new();
        let mut to_remove_orders = Vec::new();

        for changes in diff.get_changes::<Box<dyn Order>>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    to_add_orders.push(changes.new_value.unwrap());
                }
                DiffType::Modified => {
                    to_update_orders.push(changes.new_value.unwrap());
                }
                DiffType::Removed => {
                    to_remove_orders.push(changes.old_value.unwrap());
                }
            }
        }

        let to_add_order_pack: OrderPack = to_add_orders.into();
        let to_update_order_pack: OrderPack = to_update_orders.into();
        let to_remove_order_pack: OrderPack = to_remove_orders.into();

        to_add_order_pack
            .into_active_model()
            .insert_or_update_all(&txn)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        to_update_order_pack
            .into_active_model()
            .insert_or_update_all(&txn)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        to_remove_order_pack
            .delete_all(&txn)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        for changes in diff.get_changes::<Transaction>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    panic!("Aggregate root transaction should not have diff type of: Added")
                }
                DiffType::Modified => {
                    let new = changes.new_value.unwrap();

                    crate::models::transaction::Entity::update(
                        TransactionDataConverter::transform_to_do_transaction_only(&new),
                    )
                    .exec(&txn)
                    .await
                    .map_err(|e| RepositoryError::Db(e.into()))?;
                }
                DiffType::Removed => {
                    panic!("Aggregate root transaction should not have diff type of: Removed")
                }
            }
        }

        txn.commit().await.context("Failed to commit transaction")?;

        Ok(())
    }

    async fn on_delete(&self, aggregate: Transaction) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::transaction::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .map_err(|e| RepositoryError::Db(e.into()))?;
        }

        Ok(())
    }
}

#[async_trait]
impl TransactionRepository for TransactionRepositoryImpl {
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Transaction>, RepositoryError> {
        let r = self
            .query_transaction(|q| q.filter(crate::models::transaction::Column::Uuid.eq(uuid)))
            .await?;

        Ok(r.into_iter().next())
    }

    async fn find_by_user_id(&self, user_id: UserId) -> Result<Vec<Transaction>, RepositoryError> {
        let r = self
            .query_transaction(|q| {
                q.filter(crate::models::transaction::Column::UserId.eq(user_id.to_db_value()))
            })
            .await?;

        Ok(r)
    }

    async fn get_user_balance(&self, user_id: UserId) -> Result<Option<Decimal>, RepositoryError> {
        #[derive(Debug, FromQueryResult)]
        struct Balance {
            balance: Decimal,
        }

        let r = Balance::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT "balance"."balance" FROM "balance" WHERE "balance"."user_id" = $1"#,
            [user_id.to_db_value().into()],
        ))
        .one(&self.db)
        .await
        .context(format!("Failed to query balance for user: {}", user_id))?;

        Ok(r.map(|item| item.balance))
    }
}
