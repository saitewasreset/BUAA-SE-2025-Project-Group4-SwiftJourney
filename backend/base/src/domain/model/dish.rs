//! # 火车餐实体模块
//!
//! 该模块定义了火车餐相关的实体数据结构及其相关操作。主要包含以下内容：
//!
//! - `DishTime`: 枚举类型，表示餐点的时间类型，包括午餐（Lunch）和晚餐（Dinner）。
//! - `DishTimeError`: 枚举类型，表示转换餐点时间时可能出现的错误。
//! - `Dish`: 结构体，表示火车上的餐点实体，包含餐点的唯一标识符、所属列车标识符、餐点类型、餐点时间、名称、单价以及图片列表。
//!
//! ## Examples
//!
//! ```rust
//! use base::domain::model::train::TrainId;
//! use base::domain::model::dish::{Dish, DishTime};
//! use rust_decimal::Decimal;
//!
//! let train_id = TrainId::from(1);
//! let dish = Dish::new(
//!     None,
//!     train_id,
//!     "Main Course".to_string(),
//!     DishTime::Lunch,
//!     "Grilled Salmon".to_string(),
//!     Decimal::new(3500, 2), // 35.00
//!     vec!["image1.jpg".to_string(), "image2.jpg".to_string()],
//! );
//!
//! assert_eq!(dish.name(), "Grilled Salmon");
//! assert_eq!(dish.dish_time(), DishTime::Lunch);
//! assert_eq!(dish.unit_price(), Decimal::new(3500, 2));
//! ```
//!
//! ## Errors
//!
//! 当尝试将无效字符串转换为 `DishTime` 枚举类型时，会返回 `DishTimeError::InvalidDishTime` 错误。
//!
//! ```rust
//! use base::domain::model::dish::{DishTime, DishTimeError};
//!
//! let result: Result<DishTime, DishTimeError> = "breakfast".try_into();
//! assert!(result.is_err());
//! ```
use crate::domain::model::train::TrainId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;
use rust_decimal::Decimal;
use std::fmt::Formatter;
use thiserror::Error;

define_id_type!(Dish);

/// 枚举类型，表示餐点的时间类型。
///
/// 目前支持两种餐点时间：
/// - `Lunch`: 午餐
/// - `Dinner`: 晚餐
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DishTime {
    Lunch,
    Dinner,
}

/// 枚举类型，表示转换餐点时间时可能出现的错误。
///
/// 目前只有一种错误类型：
/// - `InvalidDishTime`: 尝试将无效字符串转换为 `DishTime` 枚举类型时抛出此错误。
#[derive(Debug, Error)]
pub enum DishTimeError {
    #[error("Invalid dish time: {0}")]
    InvalidDishTime(String),
}

impl TryFrom<&str> for DishTime {
    type Error = DishTimeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lunch" => Ok(DishTime::Lunch),
            "dinner" => Ok(DishTime::Dinner),
            x => Err(DishTimeError::InvalidDishTime(x.to_string())),
        }
    }
}

impl From<DishTime> for &str {
    fn from(value: DishTime) -> Self {
        match value {
            DishTime::Lunch => "lunch",
            DishTime::Dinner => "dinner",
        }
    }
}

impl From<&DishTime> for &str {
    fn from(value: &DishTime) -> Self {
        match &value {
            DishTime::Lunch => "lunch",
            DishTime::Dinner => "dinner",
        }
    }
}

impl std::fmt::Display for DishTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&DishTime as Into<&str>>::into(self))
    }
}

/// 表示火车上的餐点实体。
///
/// 包含以下字段：
/// - `id`: 餐点的唯一标识符，可以为空。
/// - `train_id`: 所属列车的标识符。
/// - `dish_type`: 餐点类型。
/// - `dish_time`: 餐点时间，可以是午餐或晚餐。
/// - `name`: 餐点名称。
/// - `unit_price`: 餐点单价。
/// - `images`: 餐点图片的列表。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dish {
    id: Option<DishId>,
    train_id: TrainId,
    dish_type: String,
    dish_time: DishTime,
    name: String,
    unit_price: Decimal,
    images: Vec<String>,
}

impl Dish {
    /// 创建一个新的 `Dish` 实例。
    ///
    /// Arguments：
    /// - `id`: 餐点的唯一标识符，可以为空。
    /// - `train_id`: 所属列车的标识符。
    /// - `dish_type`: 餐点类型。
    /// - `dish_time`: 餐点时间。
    /// - `name`: 餐点名称。
    /// - `unit_price`: 餐点单价。
    /// - `images`: 餐点图片的列表。
    ///
    /// Returns：
    /// - 新创建的 `Dish` 实例。
    pub fn new(
        id: Option<DishId>,
        train_id: TrainId,
        dish_type: String,
        dish_time: DishTime,
        name: String,
        unit_price: Decimal,
        images: Vec<String>,
    ) -> Self {
        Self {
            id,
            train_id,
            dish_type,
            dish_time,
            name,
            unit_price,
            images,
        }
    }

    /// 获取餐点所属的列车标识符。
    ///
    /// Returns：
    /// - 所属列车的标识符。
    pub fn train_id(&self) -> TrainId {
        self.train_id
    }

    /// 获取餐点类型。
    ///
    /// Returns：
    /// - 餐点类型的字符串引用。
    pub fn dish_type(&self) -> &str {
        &self.dish_type
    }

    /// 获取餐点时间。
    ///
    /// Returns：
    /// - 餐点时间枚举类型。
    pub fn dish_time(&self) -> DishTime {
        self.dish_time
    }

    /// 获取餐点名称。
    ///
    /// Returns：
    /// - 餐点名称的字符串引用。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取餐点单价。
    ///
    /// Returns：
    /// - 餐点单价。
    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    /// 获取餐点图片列表。
    ///
    /// Returns：
    /// - 餐点图片列表的不可变引用。
    pub fn images(&self) -> &[String] {
        &self.images
    }
}

impl Identifiable for Dish {
    type ID = DishId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id)
    }
}

impl Entity for Dish {}
impl Aggregate for Dish {}
