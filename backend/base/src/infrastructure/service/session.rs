//! 会话管理服务实现层
//!
//! 提供`SessionManagerService` trait的具体实现，处理会话管理的实际业务逻辑。
//!
//! 主要包含：
//! - `SessionManagerServiceImpl`：会话管理服务的线程安全实现
//!
//! 特性：
//! - 基于DashMap的并发控制
//! - 自动会话淘汰机制
//! - 配置驱动的会话管理策略

use std::{collections::VecDeque, sync::Arc};

use chrono::Utc;
use dashmap::DashMap;

use crate::domain::{
    RepositoryError,
    model::{
        session::{Session, SessionId},
        session_config::SessionConfig,
        user::UserId,
    },
    repository::session::SessionRepository,
    service::session::SessionManagerService,
};

/// 会话管理服务实现
///
/// 提供线程安全的会话管理功能，主要特点：
/// 1. 自动维护用户-会话映射关系
/// 2. 实现最大会话数限制
/// 3. 基于配置的TTL管理
///
/// # 线程安全
/// 使用`Arc<DashMap>`保证线程安全，适合多线程环境。
pub struct SessionManagerServiceImpl<R>
where
    R: SessionRepository,
{
    /// 会话存储仓库
    session_repository: R,
    /// 会话管理配置
    session_config: SessionConfig,
    /// 用户ID到会话ID的映射
    user_id_to_session: Arc<DashMap<UserId, VecDeque<SessionId>>>,
}

impl<R> SessionManagerServiceImpl<R>
where
    R: SessionRepository,
{
    /// 创建新的会话管理器
    ///
    /// # 参数
    /// - `session_repository`: 会话存储实现
    /// - `config`: 会话配置参数
    pub fn new(session_repository: R, config: SessionConfig) -> Self {
        Self {
            session_repository,
            session_config: config,
            user_id_to_session: Arc::new(DashMap::new()),
        }
    }
}

impl<R> SessionManagerService for SessionManagerServiceImpl<R>
where
    R: SessionRepository,
{
    /// 创建新会话(自动管理会话限制)
    ///
    /// 当用户会话数超过配置限制时，会自动淘汰最旧的会话。
    async fn create_session(&self, user_id: UserId) -> Result<Session, RepositoryError> {
        let created_at = Utc::now();
        let expires_at = created_at + self.session_config.default_ttl;

        // 会话淘汰逻辑
        if let Some(mut sessions) = self.user_id_to_session.get_mut(&user_id) {
            if sessions.len() >= self.session_config.max_concurrent_sessions_per_user {
                if let Some(evicted_session_id) = sessions.pop_front() {
                    if let Some(evicted_session) =
                        self.session_repository.find(evicted_session_id).await?
                    {
                        self.session_repository.remove(evicted_session).await?;
                    }
                }
            }
        }

        let session = Session::new(user_id, created_at, expires_at);

        // 更新会话映射
        self.user_id_to_session
            .entry(user_id)
            .or_default()
            .push_back(session.session_id());

        self.session_repository.save(session.clone()).await?;
        Ok(session)
    }

    /// 删除指定会话
    async fn delete_session(&self, session: Session) -> Result<(), RepositoryError> {
        self.session_repository.remove(session).await
    }

    /// 查询会话详情
    ///
    /// # Returns
    /// - `Some(Session)`: 找到有效会话
    /// - `None`: 会话不存在或已过期
    async fn get_session(&self, session_id: SessionId) -> Result<Option<Session>, RepositoryError> {
        self.session_repository.find(session_id).await
    }

    /// 通过会话ID获取关联用户ID
    async fn get_user_id_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<UserId>, RepositoryError> {
        self.session_repository
            .find(session_id)
            .await
            .map(|session| session.map(|s| s.user_id()))
    }
}
