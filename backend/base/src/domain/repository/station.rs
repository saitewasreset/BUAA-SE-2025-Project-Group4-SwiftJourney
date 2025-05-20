//! 火车站仓储抽象层
//!
//! 定义火车站(Station)的仓储接口。
//!
//! 主要包含：
//! - 火车站仓储接口(`StationRepository`)
//!
//! 注意：具体实现应放在基础设施层(`infrastructure::repository`)。
use crate::domain::model::city::CityId;
use crate::domain::model::station::Station;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::StationData;

/// 火车站仓储接口
///
/// 继承自通用`Repository<Station>` trait，提供火车站实体的持久化抽象。
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
/// - `load`: 加载所有火车站信息。
/// - `find_by_city`: 根据城市ID查找火车站。
/// - `find_by_name`: 根据车站名称查找火车站。
/// - `save_raw`: 保存原始火车站数据。
///
/// # Errors
///
/// 如果发生错误（如数据库连接失败），将返回`RepositoryError`。
#[async_trait]
pub trait StationRepository: Repository<Station> + 'static + Send + Sync {
    /// 加载所有火车站信息。
    ///
    /// # Returns
    /// 成功时返回所有火车站的列表；失败时返回`RepositoryError`。
    async fn load(&self) -> Result<Vec<Station>, RepositoryError>;

    /// 根据城市ID查找火车站。
    ///
    /// # Arguments
    /// * `city_id` - 要查找的城市ID。
    ///
    /// # Returns
    /// 成功时返回匹配的火车站列表；失败时返回`RepositoryError`。
    async fn find_by_city(&self, city_id: CityId) -> Result<Vec<Station>, RepositoryError>;

    /// 根据车站名称查找火车站。
    ///
    /// # Arguments
    /// * `station_name` - 要查找的车站名称。
    ///
    /// # Returns
    /// 成功时返回匹配的火车站（如果有）；失败时返回`RepositoryError`。
    async fn find_by_name(&self, station_name: &str) -> Result<Option<Station>, RepositoryError>;

    /// 保存原始火车站数据。
    ///
    /// # Arguments
    /// * `station_data` - 原始火车站数据。
    ///
    /// # Returns
    /// 成功时返回`()`；失败时返回`RepositoryError`。
    async fn save_raw(&self, station_data: StationData) -> Result<(), RepositoryError>;
}
