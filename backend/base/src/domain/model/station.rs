//! 火车站领域模型模块
//!
//! 该模块定义了火车票订购系统中的火车站相关实体。
//! 车站作为系统核心领域概念，表示列车可以停靠的物理站点。
//!
//! # 核心概念
//! - [`Station`][]: 火车站聚合根，包含车站基本信息和所属城市
//! - [`StationId`][]: 车站唯一标识符
//! - 与[`crate::domain::model::route::Stop`]的区别：Station是静态站点信息，Stop是车次特定停靠信息
//!
//! # 关键特性
//! - 每个车站必须属于一个城市
//! - 作为聚合根维护车站数据的一致性
//! - 与路线(Route)模块协同工作
//!
//! # Examples
//!
//! 创建车站实例：
//! ```
//! # use base::domain::model::station::{Station, StationId};
//! # use base::domain::model::city::CityId;
//! let station = Station::new(
//!     Some(StationId::from(1u64)),
//!     "北京南站".to_string(),
//!     CityId::from(101u64) // 北京市
//! );
//! ```
//!
//! 获取车站信息：
//! ```
//! # use base::domain::model::station::{Station, StationId};
//! # use base::domain::model::city::CityId;
//! # let station = Station::new(
//! #    Some(StationId::from(1u64)),
//! #    "北京南站".to_string(),
//! #    CityId::from(101u64)
//! # );
//! assert_eq!(station.name(), "北京南站");
//! assert_eq!(station.city_id(), CityId::from(101u64));
//! ```
use crate::domain::model::city::CityId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;

define_id_type!(Station);

/// 火车站聚合根
///
/// 表示火车票系统中的物理车站，包含车站名称和所属城市信息。
/// 与[`crate::domain::model::route::Stop`]的区别在于，Station是静态的站点定义，而Stop是车次特定的停靠记录。
///
/// # 字段说明
/// - `station_id`: 车站唯一标识
/// - `name`: 车站名称(如"北京南站")
/// - `city_id`: 所属城市ID
///
/// # 不变量
/// - 必须属于有效的城市
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Station {
    station_id: Option<StationId>,
    name: String,
    city_id: CityId,
}

impl Identifiable for Station {
    type ID = StationId;

    /// 获取车站ID
    ///
    /// # Returns
    /// 返回`Some(StationId)`如果存在，否则`None`
    fn get_id(&self) -> Option<Self::ID> {
        self.station_id
    }

    /// 设置车站ID
    fn set_id(&mut self, id: Self::ID) {
        self.station_id = Some(id);
    }
}

impl Entity for Station {}
impl Aggregate for Station {}

impl Station {
    /// 创建新车站实例
    ///
    /// # Arguments
    /// * `station_id`: 车站ID(新建时可为None)
    /// * `name`: 车站名称
    /// * `city_id`: 所属城市ID
    pub fn new(station_id: Option<StationId>, name: String, city_id: CityId) -> Self {
        Station {
            station_id,
            name,
            city_id,
        }
    }

    /// 获取车站名称
    ///
    /// # Returns
    /// 返回车站名称的字符串切片
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取所属城市ID
    ///
    /// # Returns
    /// 返回城市ID的拷贝
    pub fn city_id(&self) -> CityId {
        self.city_id
    }
}
