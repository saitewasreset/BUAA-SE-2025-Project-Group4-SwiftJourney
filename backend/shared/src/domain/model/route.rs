//! 车次路线领域模型模块
//!
//! 该模块定义了火车票订购系统中的车次路线相关实体和值对象。
//! 路线表示一个车次从始发站到终到站的完整运行路径，包含沿途所有停靠站点信息。
//!
//! # 核心概念
//! - [`Route`][]: 车次路线聚合根，包含完整的站点停靠序列
//! - [`Stop`][]: 车次在特定站点的停靠记录，包含到达/出发时间等信息
//! - [`RouteId`]/[`StopId`]: 路线和停靠记录的唯一标识符
//!
//! # 关键特性
//! - 停靠时间使用相对于始发站发车时间的秒数表示
//! - 停靠顺序由`order`字段严格保证
//! - 路线作为聚合根维护所有停靠记录的一致性
//!
//! # Examples
//!
//! 创建路线并添加停靠站点：
//! ```
//! use base::domain::model::route::{RouteId, StopId, Route, Stop};
//! use base::domain::model::station::{StationId};
//!
//! let mut route = Route::new(Some(RouteId::from(1u64)));
//! route.add_stop(
//!     Some(StopId::from(1u64)),
//!     StationId::from(101u64), // 北京站
//!     0,     // 始发站到达时间(发车时间)
//!     120,   // 北京站出发时间(2分钟后)
//!     1      // 第一站
//! );
//! route.add_stop(
//!     Some(StopId::from(2u64)),
//!     StationId::from(102u64), // 天津站
//!     1800,  // 30分钟后到达
//!     1860,  // 31分钟后发车
//!     2      // 第二站
//! );
//! ```
//!
//! 获取路线信息：
//! ```
//! # use base::domain::model::route::{RouteId, StopId, Route, Stop};
//! # let route = Route::new(Some(RouteId::from(1u64)));
//! let stops = route.stops();
//! assert_eq!(stops.len(), 0); // 初始无停靠站
//! ```
use crate::domain::model::station::StationId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;

define_id_type!(Stop);

/// 车次在特定站点的停靠记录
///
/// 记录车次在某个站点的到达时间、出发时间和停靠顺序等信息。
/// 与[`crate::domain::model::station::Station`]的区别在于，Stop是车次特定的运行时信息。
///
/// # 字段说明
/// - `stop_id`: 停靠记录唯一标识
/// - `route_id`: 所属路线ID
/// - `station_id`: 车站ID
/// - `arrival_time`: 到达时间(相对于始发站发车的秒数)
/// - `departure_time`: 出发时间(相对于始发站发车的秒数)
/// - `order`: 停靠顺序(从0开始)
///
/// # 不变量
/// - `arrival_time` <= `departure_time`
/// - `order`必须大于等于0且在该路线内唯一
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stop {
    stop_id: Option<StopId>,
    route_id: Option<RouteId>,
    station_id: StationId,
    arrival_time: u32,
    departure_time: u32,
    order: u32,
}

impl Identifiable for Stop {
    type ID = StopId;

    fn get_id(&self) -> Option<Self::ID> {
        self.stop_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.stop_id = Some(id)
    }
}

impl Entity for Stop {}

impl Stop {
    /// 创建新的停靠记录
    ///
    /// # Arguments
    /// * `id`: 停靠记录ID(新建时可为None)
    /// * `route_id`: 所属路线ID
    /// * `station_id`: 车站ID
    /// * `arrival_time`: 到达时间(秒)
    /// * `departure_time`: 出发时间(秒)
    /// * `order`: 停靠顺序
    pub fn new(
        id: Option<StopId>,
        route_id: Option<RouteId>,
        station_id: StationId,
        arrival_time: u32,
        departure_time: u32,
        order: u32,
    ) -> Self {
        Stop {
            stop_id: id,
            route_id,
            station_id,
            arrival_time,
            departure_time,
            order,
        }
    }

    /// 获取车站ID
    pub fn station_id(&self) -> StationId {
        self.station_id
    }

    /// 获取到达时间(秒)
    pub fn arrival_time(&self) -> u32 {
        self.arrival_time
    }

    /// 获取出发时间(秒)
    pub fn departure_time(&self) -> u32 {
        self.departure_time
    }

    /// 获取停靠顺序
    pub fn order(&self) -> u32 {
        self.order
    }
}

define_id_type!(Route);
/// 车次路线聚合根
///
/// 表示一个车次的完整运行路线，包含所有停靠站点信息。
/// 作为聚合根负责维护停靠记录的一致性和顺序。
///
/// # 字段说明
/// - `route_id`: 路线唯一标识
/// - `stops`: 停靠站点集合(按order排序)
///
/// # 不变量
/// - 停靠记录必须按order字段严格排序
/// - 始发站必须有arrival_time=0、departure_time=0
/// - 终到站必须有departure_time=0
/// - 相邻站点时间必须合理(前站出发<=后站到达)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    route_id: Option<RouteId>,
    stops: Vec<Stop>,
}

impl Identifiable for Route {
    type ID = RouteId;

    fn get_id(&self) -> Option<Self::ID> {
        self.route_id
    }

    /// 设置路线ID并同步更新所有停靠记录的route_id
    fn set_id(&mut self, id: Self::ID) {
        self.route_id = Some(id);

        for stop in &mut self.stops {
            stop.route_id = Some(id);
        }
    }
}

impl Entity for Route {}

impl Aggregate for Route {}

impl Route {
    /// 创建新路线
    ///
    /// # Arguments
    /// * `id`: 路线ID(新建时可为None)
    pub fn new(id: Option<RouteId>) -> Self {
        Route {
            route_id: id,
            stops: Vec::new(),
        }
    }

    /// 获取停靠记录切片(按order排序)
    pub fn stops(&self) -> &[Stop] {
        &self.stops
    }

    /// 获取停靠记录迭代器(按order排序)
    pub fn stop_pairs(&self) -> impl Iterator<Item = (&Stop, &Stop)> {
        self.stops.windows(2).map(|pair| (&pair[0], &pair[1]))
    }

    /// 消费self返回所有停靠记录
    pub fn into_stops(self) -> Vec<Stop> {
        self.stops
    }

    /// 添加停靠站点
    ///
    /// # Arguments
    /// * `stop_id`: 停靠记录ID
    /// * `station_id`: 车站ID
    /// * `arrival_time`: 到达时间(秒)
    /// * `departure_time`: 出发时间(秒)
    /// * `order`: 停靠顺序
    pub fn add_stop(
        &mut self,
        stop_id: Option<StopId>,
        station_id: StationId,
        arrival_time: u32,
        departure_time: u32,
        order: u32,
    ) {
        let stop = Stop::new(
            stop_id,
            self.route_id,
            station_id,
            arrival_time,
            departure_time,
            order,
        );
        self.stops.push(stop);
    }
}
