//! 会话仓储抽象层
//!
//! 定义会话(Session)的仓储接口和配置，遵循领域驱动设计中的仓储模式(Repository Pattern)。
//!
//! 主要包含：
//! - 会话仓储配置(`SessionRepositoryConfig`)
//! - 会话仓储接口(`SessionRepository`)
//!
//! 注意：具体实现应放在基础设施层(`infrastructure::repository`)。
use crate::domain::Repository;
use crate::domain::model::session::Session;

/// 会话仓储配置参数
///
/// 用于控制会话仓储的实现行为，如清理过期会话的频率等。
/// 实现默认特征提供合理的默认值。
///
/// # Examples
///
/// ```
/// use base::domain::repository::session::SessionRepositoryConfig;
/// use std::time::Duration;
///
/// // 使用默认配置(5分钟清理一次)
/// let default_config = SessionRepositoryConfig::default();
///
/// // 自定义配置
/// let custom_config = SessionRepositoryConfig {
///     session_cleanup_interval: Duration::from_secs(60), // 1分钟清理一次
/// };
/// ```
pub struct SessionRepositoryConfig {
    /// 清理过期会话的时间间隔
    pub session_cleanup_interval: std::time::Duration,
}

impl Default for SessionRepositoryConfig {
    fn default() -> Self {
        SessionRepositoryConfig {
            session_cleanup_interval: std::time::Duration::from_secs(60 * 5),
        }
    }
}

/// 会话仓储接口
///
/// 继承自通用`Repository<Session>` trait，提供会话实体的持久化抽象。
/// 具体实现可能包括：
/// - 内存存储
/// - 数据库存储
/// - Redis存储等
///
/// # 实现说明
///
/// 实现者应确保：
/// 1. 线程安全
/// 2. 正确处理并发访问
/// 3. 遵循配置参数
///
/// # Notes
///
/// 默认情况下，通过`Repository<Session>` trait已提供基本CRUD操作，
/// 如需扩展特殊查询方法，可在此trait中添加。
pub trait SessionRepository: Repository<Session> {}
