#![cfg(test)]

use async_trait::async_trait;

use crate::domain::model::session::{Session, SessionId};
use crate::domain::model::user::UserId;
use crate::domain::service::session::SessionManagerService;
use crate::domain::RepositoryError;
use mockall::mock;

// 自动生成 Mock 类型
mock! {
    pub SessionManagerService {}

    #[async_trait]
    impl SessionManagerService for SessionManagerService {
        async fn create_session(&self, user_id: UserId) -> Result<Session, RepositoryError>;
        async fn delete_session(&self, session: Session) -> Result<(), RepositoryError>;
        async fn get_session(&self, session_id: SessionId) -> Result<Option<Session>, RepositoryError>;
        async fn get_user_id_by_session(&self, session_id: SessionId) -> Result<Option<UserId>, RepositoryError>;
        async fn verify_session_id(&self, session_id_str: &str) -> Result<bool, RepositoryError>;
    }
}

// helper 构造函数，方便测试快速创建
pub fn mock_session_service() -> MockSessionManagerService {
    MockSessionManagerService::new()
}
