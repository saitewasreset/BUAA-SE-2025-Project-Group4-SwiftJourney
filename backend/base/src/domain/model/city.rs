//! 城市领域模型模块
//!
//! 该模块定义了火车票订购系统中的城市相关实体和值对象。
//! 城市作为系统核心领域概念，用于表示列车停靠站点的地理位置信息。
//!
//! # 主要结构
//! - [`City`][]: 城市聚合根，包含城市基本信息和所属省份
//! - [`CityName`][]: 城市名称值对象
//! - [`ProvinceName`][]: 省份名称值对象
//! - [`CityId`][]: 城市唯一标识符(通过宏生成)
//!
//! # 领域特征实现
//! - 实现了[`Identifiable`]、[`Entity`]和[`Aggregate`]特征
//! - 作为聚合根维护城市数据的一致性边界
//!
//! # Examples
//!
//! 创建城市实例：
//! ```
//! # use base::domain::model::city::{City, CityName, ProvinceName, CityId};
//!
//! let city = City::new(
//!     Some(CityId::from(1u64)),
//!     CityName::from("北京市".to_string()),
//!     ProvinceName::from("北京市".to_string())
//! );
//! ```
//!
//! 获取城市信息：
//! ```
//! # use base::domain::model::city::{City, CityName, ProvinceName, CityId};
//! # use std::ops::Deref;
//! # let city = City::new(
//! #    Some(CityId::from(1u64)),
//! #    CityName::from("北京".to_string()),
//! #    ProvinceName::from("北京市".to_string())
//! # );
//! assert_eq!(city.name().deref(), "北京市");
//! assert_eq!(city.province().deref(), "北京市");
//! ```
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;
use std::ops::Deref;

define_id_type!(City);

/// 城市名称值对象
///
/// 封装城市名称字符串，提供类型安全和相关行为。
/// 实现了[`Deref`]以方便作为字符串切片使用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CityName(String);

impl From<String> for CityName {
    /// 从字符串创建城市名称
    ///
    /// # Arguments
    /// * `value` - 城市名称字符串
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for CityName {
    type Target = str;
    /// 解引用为字符串切片
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 省份名称值对象
///
/// 封装省份名称字符串，提供类型安全和相关行为。
/// 实现了[`Deref`]以方便作为字符串切片使用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProvinceName(String);

impl Deref for ProvinceName {
    type Target = str;

    /// 解引用为字符串切片
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ProvinceName {
    /// 从字符串创建省份名称
    ///
    /// # Arguments
    /// * `value` - 省份名称字符串
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// 城市聚合根
///
/// 表示火车票订购系统中的城市实体，包含：
/// - 唯一标识符(可选，新建时可能为None)
/// - 城市名称
/// - 所属省份名称
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct City {
    city_id: Option<CityId>,
    name: CityName,
    province: ProvinceName,
}

impl Identifiable for City {
    type ID = CityId;

    /// 获取城市ID
    ///
    /// # Returns
    /// 返回`Some(CityId)`如果存在，否则`None`
    fn get_id(&self) -> Option<Self::ID> {
        self.city_id
    }

    /// 设置城市ID
    ///
    /// # Arguments
    /// * `id` - 要设置的CityId
    fn set_id(&mut self, id: Self::ID) {
        self.city_id = Some(id)
    }
}

impl Entity for City {}
impl Aggregate for City {}

impl City {
    /// 创建新的城市实例
    ///
    /// # Arguments
    /// * `city_id` - 可选的城市ID
    /// * `name` - 城市名称
    /// * `province` - 所属省份名称
    ///
    /// # Returns
    /// 返回新构建的城市实例
    pub fn new(city_id: Option<CityId>, name: CityName, province: ProvinceName) -> Self {
        City {
            city_id,
            name,
            province,
        }
    }

    /// 获取城市名称引用
    ///
    /// # Returns
    /// 返回城市名称的不可变引用
    pub fn name(&self) -> &CityName {
        &self.name
    }

    /// 获取省份名称引用
    ///
    /// # Returns
    /// 返回省份名称的不可变引用
    pub fn province(&self) -> &ProvinceName {
        &self.province
    }
}
