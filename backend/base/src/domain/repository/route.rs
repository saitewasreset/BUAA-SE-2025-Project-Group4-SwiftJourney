//! 路线仓储抽象层
//!
//! 定义车次路线(Route)的仓储接口。
//!
//! 主要包含：
//! - 路线仓储接口(`RouteRepository`)
//!
//! 注意：具体实现应放在基础设施层(`infrastructure::repository`)。
use crate::domain::model::route::{Route, RouteId};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::RouteStationInfo;

/// 路线仓储接口
///
/// 继承自通用`Repository<Route>` trait，提供车次路线实体的持久化抽象。
/// 具体实现可能包括：
/// - 内存存储
/// - 数据库存储
/// - 缓存存储等
///
/// # 实现说明
///
/// 实现者应确保：
/// 1. 线程安全
/// 2. 正确处理并发访问
/// 3. 支持异步操作
///
/// # 方法
///
/// - `load`: 加载所有车次路线信息。
/// - `save_raw`: 保存原始车次路线数据。
///
/// # Errors
///
/// 如果发生错误（如数据库连接失败），将返回`RepositoryError`。
#[async_trait]
pub trait RouteRepository: Repository<Route> {
    /// 加载所有车次路线信息。
    ///
    /// # Returns
    /// 成功时返回所有车次路线的列表；失败时返回`RepositoryError`。
    async fn load(&self) -> Result<Vec<Route>, RepositoryError>;

    /// 保存原始车次路线数据。
    ///
    /// # Arguments
    /// * `raw_routes` - 原始车次路线数据。
    ///
    /// # Returns
    /// 成功时返回新保存的车次路线ID；失败时返回`RepositoryError`。
    async fn save_raw(&self, raw_routes: Vec<RouteStationInfo>)
    -> Result<RouteId, RepositoryError>;
}
