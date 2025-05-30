//! 列车车次仓储抽象层
//!
//! 定义列车车次(Train)的仓储接口。
//!
//! 主要包含：
//! - 列车车次仓储接口(`TrainRepository`)
//!
//! 注意：具体实现应放在基础设施层(`infrastructure::repository`)。
use crate::Verified;
use crate::domain::model::train::{SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::SeatId;
use crate::domain::repository::route::RouteRepository;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use shared::data::{TrainNumberData, TrainTypeData};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 列车车次仓储接口
///
/// 继承自通用`Repository<Train>` trait，提供列车车次实体的持久化抽象。
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
/// - `get_verified_train_number`: 获取所有已验证的车次编号。
/// - `get_verified_train_type`: 获取所有已验证的列车类型。
/// - `get_verified_seat_type`: 获取指定车次的所有已验证座位类型。
/// - `get_trains`: 加载所有列车车次信息。
/// - `get_seat_id_map`: 获取指定车次的座位ID映射。
/// - `find_by_train_number`: 根据车次编号查找列车车次。
/// - `find_by_train_type`: 根据列车类型查找列车车次。
/// - `save_raw_train_number`: 保存原始车次编号数据。
/// - `save_raw_train_type`: 保存原始列车类型数据。
///
/// # Errors
///
/// 如果发生错误（如数据库连接失败），将返回`RepositoryError`。
#[async_trait]
pub trait TrainRepository: Repository<Train> + 'static + Send + Sync {
    /// 获取所有已验证的车次编号（字符串）。
    ///
    /// # Returns
    /// 成功时返回已验证的车次编号（字符串）的集合；失败时返回`RepositoryError`。
    async fn get_verified_train_number(&self) -> Result<HashSet<String>, RepositoryError>;
    /// 获取所有已验证的列车类型（字符串）。
    ///
    /// # Returns
    /// 成功时返回已验证的列车类型（字符串）的集合；失败时返回`RepositoryError`。
    async fn get_verified_train_type(&self) -> Result<HashSet<String>, RepositoryError>;

    /// 获取指定车次的所有已验证座位类型（字符串）。
    ///
    /// # Arguments
    /// * `train_id` - 车次ID。
    ///
    /// # Returns
    /// 成功时返回已验证的座位类型（字符串）的集合；失败时返回`RepositoryError`。
    async fn get_verified_seat_type(
        &self,
        train_id: TrainId,
    ) -> Result<HashSet<String>, RepositoryError>;

    /// 加载所有列车车次信息。
    ///
    /// # Returns
    /// 成功时返回所有列车车次的列表；失败时返回`RepositoryError`。
    async fn get_trains(&self) -> Result<Vec<Train>, RepositoryError>;

    /// 获取指定车次的座位ID映射。
    ///
    /// # Arguments
    /// * `train_id` - 车次ID。
    ///
    /// # Returns
    /// 成功时返回座位类型名到座位ID列表的映射；失败时返回`RepositoryError`。
    async fn get_seat_id_map(
        &self,
        train_id: TrainId,
    ) -> Result<HashMap<SeatTypeName<Verified>, Vec<SeatId>>, RepositoryError>;

    /// 根据车次编号查找列车车次。
    ///
    /// # Arguments
    /// * `train_number` - 已验证的车次编号。
    ///
    /// # Returns
    /// 成功时返回匹配的列车车次；失败时返回`RepositoryError`。
    async fn find_by_train_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Train, RepositoryError>;

    /// 根据列车类型查找列车车次。
    ///
    /// # Arguments
    /// * `train_type` - 已验证的列车类型。
    ///
    /// # Returns
    /// 成功时返回匹配的列车车次列表；失败时返回`RepositoryError`。
    async fn find_by_train_type(
        &self,
        train_type: TrainType<Verified>,
    ) -> Result<Vec<Train>, RepositoryError>;

    /// 保存原始车次编号数据。
    ///
    /// # Arguments
    /// * `train_number_data` - 原始车次编号数据。
    /// * `route_repository` - 路线仓储实例。
    ///
    /// # Returns
    /// 成功时返回`()`；失败时返回`RepositoryError`。
    async fn save_raw_train_number<T: RouteRepository>(
        &self,
        train_number_data: TrainNumberData,
        route_repository: Arc<T>,
    ) -> Result<(), RepositoryError>;

    /// 保存原始列车类型数据。
    ///
    /// # Arguments
    /// * `train_type_data` - 原始列车类型数据。
    ///
    /// # Returns
    /// 成功时返回`()`；失败时返回`RepositoryError`。
    async fn save_raw_train_type(
        &self,
        train_type_data: TrainTypeData,
    ) -> Result<(), RepositoryError>;
}
