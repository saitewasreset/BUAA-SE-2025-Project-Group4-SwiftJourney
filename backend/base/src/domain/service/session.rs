//! 会话领域服务层
//!
//! 定义会话管理的核心领域服务接口，封装会话生命周期相关的业务逻辑。
//!
//! 主要包含：
//! - 会话管理服务接口(`SessionManagerService`)
//!
//! 注意：具体实现应放在基础设施层(`infrastructure::service`)。

use crate::domain::RepositoryError;
use crate::domain::model::session::{Session, SessionId};
use crate::domain::model::user::UserId;
use async_trait::async_trait;

/// 会话管理服务接口
///
/// 提供会话管理的核心业务操作，包括：
/// - 会话创建/删除
/// - 会话查询
/// - 会话有效性验证
///
/// # 设计原则
///
/// 1. 所有方法返回`Future`，支持异步操作
/// 2. 错误类型统一使用`RepositoryError`
/// 3. 方法命名体现业务意图(而非持久化细节)
///
/// # Notes
///
/// - 实现者应保证线程安全
#[async_trait]
pub trait SessionManagerService: 'static + Send + Sync {
    /// 创建新会话
    ///
    /// # 参数
    /// - `user_id`: 要创建会话的用户ID
    ///
    /// # 返回
    /// - `Ok(Session)`: 包含新创建的会话
    /// - `Err(RepositoryError)`: 如果创建失败
    ///
    /// # 业务规则
    /// - 自动生成会话ID
    /// - 设置默认TTL(应在实现层配置)
    async fn create_session(&self, user_id: UserId) -> Result<Session, RepositoryError>;

    /// 删除会话
    ///
    /// # 参数
    /// - `session`: 要删除的会话实体
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err(RepositoryError)`: 如果删除失败
    async fn delete_session(&self, session: Session) -> Result<(), RepositoryError>;

    /// 查询会话详情
    ///
    /// # 参数
    /// - `session_id`: 要查询的会话ID
    ///
    /// # 返回
    /// - `Ok(Some(Session))`: 找到有效会话
    /// - `Ok(None)`: 会话不存在
    /// - `Err(RepositoryError)`: 查询过程中出错
    async fn get_session(&self, session_id: SessionId) -> Result<Option<Session>, RepositoryError>;

    /// 通过会话ID获取用户ID
    ///
    /// # 参数
    /// - `session_id`: 要查询的会话ID
    ///
    /// # 返回
    /// - `Ok(Some(UserId))`: 会话有效且找到关联用户
    /// - `Ok(None)`: 会话不存在或已过期
    /// - `Err(RepositoryError)`: 查询过程中出错
    async fn get_user_id_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<UserId>, RepositoryError>;

    async fn verify_session_id(&self, session_id_str: &str) -> Result<bool, RepositoryError>;
}
