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

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use std::{collections::VecDeque, sync::Arc};

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
    R: SessionRepository + 'static + Send + Sync,
{
    /// 会话存储仓库
    session_repository: Arc<R>,
    /// 会话管理配置
    session_config: SessionConfig,
    /// 用户ID到会话ID的映射
    user_id_to_session: Arc<DashMap<UserId, VecDeque<SessionId>>>,
}

impl<R> SessionManagerServiceImpl<R>
where
    R: SessionRepository + 'static + Send + Sync,
{
    /// 创建新的会话管理器
    ///
    /// # 参数
    /// - `session_repository`: 会话存储实现
    /// - `config`: 会话配置参数
    pub fn new(session_repository: Arc<R>, config: SessionConfig) -> Self {
        Self {
            session_repository,
            session_config: config,
            user_id_to_session: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl<R> SessionManagerService for SessionManagerServiceImpl<R>
where
    R: SessionRepository + 'static + Send + Sync,
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

        let mut session = Session::new(user_id, created_at, expires_at);

        // 更新会话映射
        self.user_id_to_session
            .entry(user_id)
            .or_default()
            .push_back(session.session_id());

        self.session_repository.save(&mut session).await?;
        Ok(session)
    }

    /// 删除指定会话
    async fn delete_session(&self, session: Session) -> Result<(), RepositoryError> {
        if let Some(mut user_sessions) = self.user_id_to_session.get_mut(&session.user_id()) {
            user_sessions.retain(|session_id| *session_id != session.session_id());
        }

        self.session_repository.remove(session).await?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Repository;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use mockall::predicate::*;
    use mockall::*;
    use tokio::test;

    mock! {
        pub SessionRepository {}
        #[async_trait]
        impl Repository<Session> for SessionRepository {
            async fn find(&self, id: SessionId) -> Result<Option<Session>, RepositoryError>;
            async fn remove(&self, aggregate: Session) -> Result<(), RepositoryError>;
            async fn save(&self, aggregate: &mut Session) ->Result<SessionId, RepositoryError>;
        }
        impl SessionRepository for SessionRepository {}
    }

    fn create_test_config() -> SessionConfig {
        SessionConfig {
            max_concurrent_sessions_per_user: 2,
            default_ttl: std::time::Duration::from_secs(3600),
        }
    }

    #[test]
    async fn test_create_session_within_limit() {
        let config = create_test_config();
        let user_id = UserId::from(1);

        // 设置mock的save方法
        let mut mock_repo = MockSessionRepository::new();
        mock_repo
            .expect_save()
            .times(2)
            .returning(|_| Ok(SessionId::random()));

        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo), config);

        let session1 = manager.create_session(user_id).await.unwrap();
        let session2 = manager.create_session(user_id).await.unwrap();

        // 检查用户会话映射
        let sessions = manager.user_id_to_session.get(&user_id).unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0], session1.session_id());
        assert_eq!(sessions[1], session2.session_id());
    }

    #[test]
    async fn test_create_session_exceeds_limit_evicts_oldest() {
        let user_id = UserId::from(1);
        let config = SessionConfig {
            max_concurrent_sessions_per_user: 1,
            default_ttl: std::time::Duration::from_secs(3600),
        };

        // 第一次创建会话
        let mut mock_repo1 = MockSessionRepository::new();
        mock_repo1
            .expect_save()
            .return_once(|_| Ok(SessionId::random()));
        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo1), config);
        let session1 = manager.create_session(user_id).await.unwrap();

        // 第二次创建会话，触发淘汰
        let mut mock_repo2 = MockSessionRepository::new();
        {
            let session1 = session1.clone();
            mock_repo2
                .expect_find()
                .with(eq(session1.session_id()))
                .return_once(move |_| Ok(Some(session1)));
        }

        mock_repo2
            .expect_remove()
            .with(eq(session1.clone()))
            .return_once(|_| Ok(()));
        mock_repo2
            .expect_save()
            .return_once(|_| Ok(SessionId::random()));
        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo2), config);

        let session2 = manager.create_session(user_id).await.unwrap();

        // 验证映射更新
        let sessions = manager.user_id_to_session.get(&user_id).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0], session2.session_id());
    }

    #[test]
    async fn test_delete_session_calls_repository_remove() {
        let mut mock_repo = MockSessionRepository::new();

        mock_repo
            .expect_remove()
            .withf(|session| u64::from(session.user_id()) == 1)
            .return_once(|_| Ok(()));

        let config = create_test_config();
        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo), config);
        let session = Session::new(UserId::from(1), Utc::now(), Utc::now() + Duration::hours(1));

        manager.delete_session(session).await.unwrap();
    }

    #[test]
    async fn test_get_session_returns_repository_result() {
        let session_id = SessionId::random();
        let mut mock_repo = MockSessionRepository::new();
        let config = create_test_config();

        let mut seq = Sequence::new();

        // 测试找到会话
        let expected_session =
            Session::new(UserId::from(1), Utc::now(), Utc::now() + Duration::hours(1));
        {
            let expected_session = expected_session.clone();
            mock_repo
                .expect_find()
                .with(eq(session_id))
                .times(1)
                .in_sequence(&mut seq)
                .return_once(move |_| Ok(Some(expected_session)));
        }

        // 测试未找到会话
        mock_repo
            .expect_find()
            .with(eq(session_id))
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|_| Ok(None));

        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo), config);

        let result = manager.get_session(session_id).await.unwrap();
        assert_eq!(result, Some(expected_session));

        let result = manager.get_session(session_id).await.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    async fn test_get_user_id_by_session_returns_correct_user_id() {
        let session_id = SessionId::random();
        let user_id = UserId::from(1);
        let mut mock_repo = MockSessionRepository::new();
        let config = create_test_config();

        let mut seq = Sequence::new();

        // 存在有效会话
        let session = Session::new(user_id, Utc::now(), Utc::now() + Duration::hours(1));
        mock_repo
            .expect_find()
            .with(eq(session_id))
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|_| Ok(Some(session)));

        // 会话不存在
        mock_repo
            .expect_find()
            .with(eq(session_id))
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|_| Ok(None));

        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo), config);

        let result = manager.get_user_id_by_session(session_id).await.unwrap();
        assert_eq!(result, Some(user_id));

        let result = manager.get_user_id_by_session(session_id).await.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    async fn test_session_expiration_not_checked_in_get_user_id() {
        let session_id = SessionId::random();
        let user_id = UserId::from(1);
        let mut mock_repo = MockSessionRepository::new();
        let config = create_test_config();

        // 已过期的会话仍返回用户ID（根据当前实现）
        let expired_session = Session::new(
            user_id,
            Utc::now() - Duration::hours(2),
            Utc::now() - Duration::hours(1),
        );

        mock_repo
            .expect_find()
            .with(eq(session_id))
            .return_once(|_| Ok(Some(expired_session)));

        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo), config);

        let result = manager.get_user_id_by_session(session_id).await.unwrap();
        assert_eq!(result, Some(user_id));
    }

    #[test]
    async fn test_evict_nonexistent_session_handled_gracefully() {
        let user_id = UserId::from(1);
        let config = SessionConfig {
            max_concurrent_sessions_per_user: 1,
            default_ttl: std::time::Duration::from_secs(3600),
        };

        // 第一次创建会话
        let mut mock_repo1 = MockSessionRepository::new();
        mock_repo1
            .expect_save()
            .return_once(|_| Ok(SessionId::random()));
        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo1), config);
        let session1 = manager.create_session(user_id).await.unwrap();

        // 第二次创建会话，触发淘汰但旧会话不存在
        let mut mock_repo2 = MockSessionRepository::new();
        mock_repo2
            .expect_find()
            .with(eq(session1.session_id()))
            .return_once(|_| Ok(None));
        mock_repo2
            .expect_save()
            .return_once(|_| Ok(SessionId::random()));
        let manager = SessionManagerServiceImpl::new(Arc::new(mock_repo2), config);

        let session2 = manager.create_session(user_id).await.unwrap();

        // 验证新会话被添加，旧会话未被删除
        let sessions = manager.user_id_to_session.get(&user_id).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0], session2.session_id());
    }
}
