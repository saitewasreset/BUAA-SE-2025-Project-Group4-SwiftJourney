//! 会话仓储实现层
//!
//! 提供基于内存的会话仓储实现，使用DashMap实现线程安全的并发访问。
//!
//! 主要特性：
//! - 自动过期会话清理
//! - 线程安全的高并发访问
//! - 配置驱动的清理策略

use crate::domain::model::session::{Session, SessionId};
use crate::domain::repository::session::{SessionRepository, SessionRepositoryConfig};
use crate::domain::{Identifiable, Repository, RepositoryError};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// 内存会话仓储实现
///
/// 使用DashMap作为底层存储，提供：
/// - 线程安全的并发访问
/// - 自动清理过期会话
/// - 基于配置的维护策略
///
/// # Implementation Details
/// - 内部使用`Arc<DashMap>`实现线程安全
/// - 后台任务定期清理过期会话
/// - 所有操作具有O(1)时间复杂度
///
/// # Performance
/// - 读操作：完全并发无锁
/// - 写操作：细粒度锁
pub struct SessionRepositoryImpl {
    /// 线程安全的会话存储
    session_map: Arc<DashMap<SessionId, Session>>,
    /// 仓储配置参数
    session_repository_config: SessionRepositoryConfig,
}

impl SessionRepositoryImpl {
    /// 创建新的内存会话仓储
    ///
    /// # Arguments
    /// - `config`: 仓储配置，包含清理间隔等参数
    ///
    /// # Notes
    /// 构造函数会立即启动后台清理任务
    pub fn new(config: SessionRepositoryConfig) -> Self {
        let repo = SessionRepositoryImpl {
            session_map: Arc::new(DashMap::new()),
            session_repository_config: config,
        };

        repo.start_cleanup_task();

        repo
    }

    /// 启动后台清理任务
    ///
    /// 根据配置的间隔定期清理过期会话
    fn start_cleanup_task(&self) {
        let session_map = Arc::clone(&self.session_map);
        let interval = self.session_repository_config.session_cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;

                session_map.retain(|_, session| !session.is_expired());
            }
        });
    }
}

#[async_trait]
impl Repository<Session> for SessionRepositoryImpl {
    /// 查找会话
    ///
    /// # Arguments
    /// - `id`: 要查找的会话ID
    ///
    /// # Returns
    /// - `Some(Session)`: 找到且未过期的会话
    /// - `None`: 会话不存在或已过期
    async fn find(&self, id: SessionId) -> Result<Option<Session>, RepositoryError> {
        Ok(self.session_map.get(&id).map(|entry| entry.clone()))
    }

    /// 删除会话
    ///
    /// # Arguments
    /// - `aggregate`: 要删除的会话实体
    ///
    /// # Errors
    /// - 永远不会返回错误（内存操作保证成功）
    async fn remove(&self, aggregate: Session) -> Result<(), RepositoryError> {
        if let Some(session_id) = aggregate.get_id() {
            self.session_map.remove(&session_id);
        }

        Ok(())
    }

    /// 保存会话
    ///
    /// # Arguments
    /// - `aggregate`: 要保存的会话实体
    ///
    /// # Returns
    /// - 会话ID
    ///
    /// # Notes
    /// - 自动覆盖同名会话
    async fn save(&self, aggregate: &mut Session) -> Result<SessionId, RepositoryError> {
        if let Some(session_id) = aggregate.get_id() {
            self.session_map.insert(session_id, aggregate.clone());
            Ok(session_id)
        } else {
            let session_id = SessionId::random();

            aggregate.set_id(session_id);

            self.session_map.insert(session_id, aggregate.clone());
            Ok(session_id)
        }
    }
}

impl SessionRepository for SessionRepositoryImpl {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::session::Session;
    use crate::domain::repository::session::SessionRepositoryConfig;
    use chrono::Utc;
    use std::time::Duration;
    use tokio::time::sleep;

    fn create_repo() -> SessionRepositoryImpl {
        SessionRepositoryImpl::new(SessionRepositoryConfig {
            // 设置较短的清理间隔，便于测试过期
            session_cleanup_interval: Duration::from_millis(50),
        })
    }

    #[tokio::test]
    async fn test_save_and_find_success() {
        let repo = create_repo();
        let mut session = Session::new(
            1u64.into(),
            Utc::now(),
            Utc::now() + Duration::from_secs(60),
        );

        // 保存
        let id = repo.save(&mut session).await.unwrap();

        // 查找成功
        let found = repo.find(id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().user_id(), 1u64.into());
    }

    #[tokio::test]
    async fn test_find_nonexistent_returns_none() {
        let repo = create_repo();

        let random_id = SessionId::random();
        let result = repo.find(random_id).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_remove_success() {
        let repo = create_repo();
        let mut session = Session::new(
            1u64.into(),
            Utc::now(),
            Utc::now() + Duration::from_secs(60),
        );

        let id = repo.save(&mut session).await.unwrap();

        // 确认存在
        assert!(repo.find(id).await.unwrap().is_some());

        // 删除
        repo.remove(session).await.unwrap();

        // 删除后查找不到
        let result = repo.find(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_ok() {
        let repo = create_repo();
        let session = Session::new(
            1u64.into(),
            Utc::now(),
            Utc::now() + Duration::from_secs(60),
        );

        // 删除不存在的 session 不会报错
        let result = repo.remove(session).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_expired_session_cleanup() {
        let repo = create_repo();
        let mut session = Session::new(
            1u64.into(),
            Utc::now(),
            Utc::now() + Duration::from_millis(30),
        );

        let id = repo.save(&mut session).await.unwrap();

        // 初始时能找到
        assert!(repo.find(id).await.unwrap().is_some());

        // 等待过期 + 清理任务运行
        sleep(Duration::from_millis(200)).await;

        // 应该被清理掉
        let result = repo.find(id).await.unwrap();
        assert!(result.is_none());
    }
}
