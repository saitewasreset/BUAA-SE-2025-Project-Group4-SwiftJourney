//! # 外卖实体模块
//!
//! 该模块定义了火车票订购系统中的外卖商户及相关实体数据结构及其相关操作。主要包含以下内容：
//!
//! - `TakeawayShop`: 结构体，表示外卖商户。
//! - `TakeawayDish`: 结构体，表示外卖菜品。
use crate::domain::model::station::StationId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;
use rust_decimal::Decimal;

define_id_type!(TakeawayShop);

/// 结构体，表示外卖商户。
///
/// 包含以下字段：
/// - `id`: 外卖商户的唯一标识符，可以为空。
/// - `name`: 外卖商户的名称。
/// - `station_id`: 所属车站的唯一标识符。
/// - `images`: 外卖商户的图片列表。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TakeawayShop {
    id: Option<TakeawayShopId>,
    name: String,
    station_id: StationId,
    images: Vec<String>,
}

impl TakeawayShop {
    /// 创建一个新的 `TakeawayShop` 实例。
    ///
    /// Arguments:
    /// - `id`: 外卖商户的唯一标识符，可以为空。
    /// - `name`: 外卖商户的名称。
    /// - `station_id`: 所属车站的唯一标识符。
    /// - `images`: 外卖商户的图片列表。
    ///
    /// Returns:
    /// - 新创建的 `TakeawayShop` 实例。
    pub fn new(
        id: Option<TakeawayShopId>,
        name: String,
        station_id: StationId,
        images: Vec<String>,
    ) -> Self {
        Self {
            id,
            name,
            station_id,
            images,
        }
    }

    /// 获取外卖商户的名称。
    ///
    /// Returns:
    /// - 外卖商户的名称的字符串引用。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取外卖商户所属车站的唯一标识符。
    ///
    /// Returns:
    /// - 所属车站的唯一标识符。
    pub fn station_id(&self) -> StationId {
        self.station_id
    }

    /// 获取外卖商户的图片列表。
    ///
    /// Returns:
    /// - 外卖商户的图片列表的不可变引用。
    pub fn images(&self) -> &[String] {
        &self.images
    }
}

impl Identifiable for TakeawayShop {
    type ID = TakeawayShopId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for TakeawayShop {}
impl Aggregate for TakeawayShop {}

define_id_type!(TakeawayDish);
/// 结构体，表示外卖菜品。
///
/// 包含以下字段：
/// - `id`: 外卖菜品的唯一标识符，可以为空。
/// - `shop_id`: 所属外卖商户的唯一标识符。
/// - `name`: 外卖菜品的名称。
/// - `dish_type`: 外卖菜品的类型。
/// - `unit_price`: 外卖菜品的单价。
/// - `images`: 外卖菜品的图片列表。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TakeawayDish {
    id: Option<TakeawayDishId>,
    shop_id: TakeawayShopId,
    name: String,
    dish_type: String,
    unit_price: Decimal,
    images: Vec<String>,
}

impl TakeawayDish {
    /// 创建一个新的 `TakeawayDish` 实例。
    ///
    /// Arguments:
    /// - `id`: 外卖菜品的唯一标识符，可以为空。
    /// - `shop_id`: 所属外卖商户的唯一标识符。
    /// - `name`: 外卖菜品的名称。
    /// - `dish_type`: 外卖菜品的类型。
    /// - `unit_price`: 外卖菜品的单价。
    /// - `images`: 外卖菜品的图片列表。
    ///
    /// Returns:
    /// - 新创建的 `TakeawayDish` 实例。
    pub fn new(
        id: Option<TakeawayDishId>,
        shop_id: TakeawayShopId,
        name: String,
        dish_type: String,
        unit_price: Decimal,
        images: Vec<String>,
    ) -> Self {
        Self {
            id,
            shop_id,
            name,
            dish_type,
            unit_price,
            images,
        }
    }

    /// 获取外卖商户的唯一标识符。
    ///
    /// Returns:
    /// - 外卖商户的唯一标识符。
    pub fn shop_id(&self) -> TakeawayShopId {
        self.shop_id
    }

    /// 获取外卖菜品的名称。
    ///
    /// Returns:
    /// - 外卖菜品的名称的字符串引用。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取外卖菜品的类型。
    ///
    /// Returns:
    /// - 外卖菜品的类型的字符串引用。
    pub fn dish_type(&self) -> &str {
        &self.dish_type
    }

    /// 获取外卖菜品的单价。
    ///
    /// Returns:
    /// - 外卖菜品的单价。
    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    /// 获取外卖菜品的图片列表。
    ///
    /// Returns:
    /// - 外卖菜品的图片列表的不可变引用。
    pub fn images(&self) -> &[String] {
        &self.images
    }
}

impl Identifiable for TakeawayDish {
    type ID = TakeawayDishId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for TakeawayDish {}
impl Aggregate for TakeawayDish {}
