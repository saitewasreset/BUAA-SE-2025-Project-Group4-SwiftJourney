//! 城市仓储抽象层
//!
//! 定义城市(City)的仓储接口。
//!
//! 主要包含：
//! - 城市仓储接口(`CityRepository`)
//!
//! 注意：具体实现应放在基础设施层(`infrastructure::repository`)。
use crate::domain::model::city::{City, ProvinceName};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::CityData;

/// 城市仓储接口
///
/// 继承自通用`Repository<City>` trait，提供城市实体的持久化抽象。
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
/// - `load`: 加载所有城市信息。
/// - `find_by_name`: 根据城市名称查找城市。
/// - `find_by_province`: 根据省份名称查找城市。
/// - `save_raw`: 保存原始城市数据。
///
/// # Errors
///
/// 如果发生错误（如数据库连接失败），将返回`RepositoryError`。
#[async_trait]
pub trait CityRepository: Repository<City> + 'static + Send + Sync {
    /// 加载所有城市信息。
    ///
    /// # Returns
    /// 成功时返回所有城市的列表；失败时返回`RepositoryError`。
    async fn load(&self) -> Result<Vec<City>, RepositoryError>;

    /// 根据城市名称查找城市。
    ///
    /// # Arguments
    /// * `city_name` - 要查找的城市名称。
    ///
    /// # Returns
    /// 成功时返回匹配的城市列表；失败时返回`RepositoryError`。
    async fn find_by_name(&self, city_name: &str) -> Result<Vec<City>, RepositoryError>;

    /// 根据省份名称查找城市。
    ///
    /// # Arguments
    /// * `province_name` - 要查找的省份名称。
    ///
    /// # Returns
    /// 成功时返回匹配的城市列表；失败时返回`RepositoryError`。
    async fn find_by_province(
        &self,
        province_name: ProvinceName,
    ) -> Result<Vec<City>, RepositoryError>;

    /// 保存原始城市数据。
    ///
    /// # Arguments
    /// * `city_data` - 原始城市数据。
    ///
    /// # Returns
    /// 成功时返回`()`；失败时返回`RepositoryError`。
    async fn save_raw(&self, city_data: CityData) -> Result<(), RepositoryError>;
}
