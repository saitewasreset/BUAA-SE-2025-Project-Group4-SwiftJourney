//! 列车车次领域模型模块
//!
//! 该模块定义了火车票订购系统中的列车车次相关实体。
//! 车次作为系统核心领域概念，表示特定类型列车的运行静态模板。
//!
//! # 核心概念
//! - [`Train`][]: 列车车次聚合根，包含车次基本信息、座位类型和默认路线
//! - [`SeatType`][]: 座位类型实体，定义不同座位的容量和票价
//! - [`TrainType`][]: 列车类型值对象(如"G"、"D"等)
//! - [`TrainNumber`][]: 车次编号值对象(如"G1234")
//!
//! # 关键特性
//! - 支持验证和未验证状态的类型安全区分
//! - 票价按乘坐站数计算(单位价格×站数)
//! - 与路线(Route)模块紧密关联
//!
//! # Examples
//!
//! 创建车次实例：
//! ```
//! # use base::domain::model::train::{Train, TrainNumber, TrainType, SeatType, TrainId, SeatTypeId, SeatTypeName};
//! # use base::domain::model::route::{RouteId};
//! # use std::collections::HashMap;
//! # use sea_orm::prelude::Decimal;
//! #
//! let seats = HashMap::from([
//!     ("商务座".to_string(), SeatType::new(
//!         Some(SeatTypeId::from(1u64)),
//!         SeatTypeName::from_unchecked("商务座".into()),
//!         30,
//!         Decimal::new(5000, 2) // 50.00 SC/站
//!     )),
//!     ("二等座".to_string(), SeatType::new(
//!         Some(SeatTypeId::from(2u64)),
//!         SeatTypeName::from_unchecked("二等座".into()),
//!         200,
//!         Decimal::new(3000, 2) // 30.00 SC/站
//!     ))
//! ]);
//!
//! let train = Train::new(
//!     Some(TrainId::from(1u64)),
//!     TrainNumber::from_unchecked("G1234".to_string()),
//!     TrainType::from_unchecked("G".to_string()),
//!     seats,
//!     RouteId::from(101u64) // 默认路线ID
//! );
//! ```
//!
//! 获取车次信息：
//! ```
//! # use base::domain::model::train::{Train, TrainNumber, TrainType, SeatType, TrainId, SeatTypeId, SeatTypeName};
//! # use base::domain::model::route::{RouteId};
//! # use std::collections::HashMap;
//! # use sea_orm::prelude::Decimal;
//! #
//! # let seats = HashMap::from([
//! #    ("商务座".to_string(), SeatType::new(
//! #        Some(SeatTypeId::from(1u64)),
//! #        SeatTypeName::from_unchecked("商务座".into()),
//! #        30,
//! #        Decimal::new(5000, 2) // 50.00 SC/站
//! #    )),
//! #    ("二等座".to_string(), SeatType::new(
//! #        Some(SeatTypeId::from(2u64)),
//! #        SeatTypeName::from_unchecked("二等座".into()),
//! #        200,
//! #        Decimal::new(3000, 2) // 30.00 SC/站
//! #     ))
//! # ]);
//! #
//! # let train = Train::new(
//! #    Some(TrainId::from(1u64)),
//! #    TrainNumber::from_unchecked("G1234".to_string()),
//! #    TrainType::from_unchecked("G".to_string()),
//! #    seats,
//! #     RouteId::from(101u64) // 默认路线ID
//! # );
//! assert_eq!(train.number(), "G1234");
//! assert_eq!(train.train_type(), "G");
//! ```
use crate::domain::model::route::RouteId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use crate::{Unverified, Verified};
use id_macro::define_id_type;
use sea_orm::prelude::Decimal;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;

/// 座位类型名称值对象
///
/// 使用类型状态模式区分已验证和未验证状态。
/// 如"商务座"、"一等座"、"二等座"等。
///
/// # 泛型参数
/// - `State`: 验证状态标记(`Verified`或`Unverified`)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeatTypeName<State = Unverified>(String, PhantomData<State>);

impl SeatTypeName {
    /// 从未验证的字符串创建已验证的座位类型名称
    ///
    /// # Safety
    /// 调用者需确保名称已通过业务规则验证
    pub fn from_unchecked(value: String) -> SeatTypeName<Verified> {
        SeatTypeName(value, PhantomData)
    }
}

impl From<String> for SeatTypeName<Unverified> {
    fn from(value: String) -> Self {
        Self(value, PhantomData)
    }
}

impl<State> Deref for SeatTypeName<State> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

define_id_type!(Train);

/// 列车车次聚合根
///
/// 表示一个列车运行静态模板，包含：
/// - 车次编号(如G1234)
/// - 列车类型(如G)
/// - 座位类型及容量配置
/// - 默认运行路线
///
/// # 字段说明
/// - `id`: 车次唯一标识
/// - `number`: 车次编号
/// - `train_type`: 列车类型
/// - `seats`: 座位类型配置(类型名→配置)
/// - `default_route_id`: 默认路线ID
///
/// # 不变条件
/// - 车次编号必须符合规范
/// - 座位容量必须>=0
/// - 单位价格必须>=0
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Train {
    id: Option<TrainId>,
    number: TrainNumber<Verified>,
    train_type: TrainType<Verified>,
    seats: HashMap<String, SeatType>,
    default_route_id: RouteId,
}

impl Identifiable for Train {
    type ID = TrainId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for Train {}
impl Aggregate for Train {}

impl Train {
    /// 创建新车次实例
    ///
    /// # Arguments
    /// * `train_id`: 车次ID(新建时可为None)
    /// * `number`: 已验证的车次编号
    /// * `train_type`: 已验证的列车类型
    /// * `seats`: 座位类型配置
    /// * `default_route_id`: 默认路线ID
    pub fn new(
        train_id: Option<TrainId>,
        number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seats: HashMap<String, SeatType>,
        default_route_id: RouteId,
    ) -> Self {
        Train {
            id: train_id,
            number,
            train_type,
            seats,
            default_route_id,
        }
    }

    /// 获取车次编号
    pub fn number(&self) -> &str {
        &self.number
    }

    /// 获取列车类型
    pub fn train_type(&self) -> &str {
        &self.train_type
    }

    /// 获取座位配置
    ///
    /// # Returns
    /// 返回座位类型名到配置的不可变引用
    pub fn seats(&self) -> &HashMap<String, SeatType> {
        &self.seats
    }

    /// 获取默认路线ID
    pub fn default_route_id(&self) -> RouteId {
        self.default_route_id
    }
}

define_id_type!(SeatType);

/// 座位类型实体
///
/// 定义特定车次可提供的座位类型及其属性。
///
/// # 字段说明
/// - `seat_type_id`: 座位类型ID
/// - `type_name`: 座位类型名称(如"商务座")
/// - `capacity`: 该类型座位总数
/// - `price`: 每站票价(单位:SC)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeatType {
    seat_type_id: Option<SeatTypeId>,
    type_name: SeatTypeName<Verified>,
    capacity: u32,
    price: Decimal,
}

impl SeatType {
    /// 创建新座位类型
    ///
    /// # Arguments
    /// * `seat_type_id`: 座位类型ID
    /// * `type_name`: 已验证的座位类型名称
    /// * `capacity`: 座位容量
    /// * `price`: 每站票价
    pub fn new(
        seat_type_id: Option<SeatTypeId>,
        type_name: SeatTypeName<Verified>,
        capacity: u32,
        price: Decimal,
    ) -> Self {
        Self {
            seat_type_id,
            type_name,
            capacity,
            price,
        }
    }

    /// 获取座位类型名称
    pub fn name(&self) -> &str {
        &self.type_name
    }

    /// 获取座位容量
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// 获取每站票价
    pub fn unit_price(&self) -> Decimal {
        self.price
    }
}

impl Identifiable for SeatType {
    type ID = SeatTypeId;

    fn get_id(&self) -> Option<Self::ID> {
        self.seat_type_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.seat_type_id = Some(id)
    }
}

impl Entity for SeatType {}

/// 列车类型值对象
///
/// 使用类型状态模式区分已验证和未验证状态。
/// 如"G"、"D"、"K"等。
///
/// # 泛型参数
/// - `State`: 验证状态标记(`Verified`或`Unverified`)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrainType<State = Unverified>(String, PhantomData<State>);

impl TrainType {
    /// 从未验证的字符串创建已验证的列车类型
    ///
    /// # Safety
    /// 调用者需确保类型已通过业务规则验证
    pub fn from_unchecked(value: String) -> TrainType<Verified> {
        TrainType(value, PhantomData)
    }
}

impl From<String> for TrainType<Unverified> {
    fn from(value: String) -> Self {
        TrainType(value, PhantomData)
    }
}

impl<T> Deref for TrainType<T> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 车次编号值对象
///
/// 使用类型状态模式区分已验证和未验证状态。
/// 如"G1234"、"D568"等。
///
/// # 泛型参数
/// - `State`: 验证状态标记(`Verified`或`Unverified`)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrainNumber<State = Unverified>(String, PhantomData<State>);

impl TrainNumber {
    /// 从未验证的字符串创建已验证的车次编号
    ///
    /// # Safety
    /// 调用者需确保编号已通过业务规则验证
    pub fn from_unchecked(value: String) -> TrainNumber<Verified> {
        TrainNumber(value, PhantomData)
    }
}

impl From<String> for TrainNumber<Unverified> {
    fn from(value: String) -> Self {
        TrainNumber(value, PhantomData)
    }
}

impl<T> Deref for TrainNumber<T> {
    type Target = str;

    /// 解引用为字符串切片
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
