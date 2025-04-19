//! 用户资料服务实现
//!
//! 本模块提供了`UserProfileService` trait的具体实现，负责处理用户资料的查询和更新操作。
//! 通过与会话管理和用户仓储的交互，完成业务逻辑的执行。
//!
//! # 主要组件
//! - `UserProfileServiceImpl`: 用户资料服务的主要实现结构体
//!   - 依赖`SessionManagerService`进行会话验证
//!   - 依赖`UserRepository`进行用户数据存取
//!
//! # 实现细节
//! 服务实现包含以下核心功能:
//! 1. 通过会话ID验证用户身份
//! 2. 查询用户资料信息
//! 3. 更新用户资料信息
//!
//! # 注意事项
//! - 所有操作都需要有效的会话ID
//! - 更新操作会验证输入数据的有效性
//! - 错误处理遵循应用层定义的错误规范
use crate::application::ApplicationError;
use crate::application::commands::user_profile::{SetUserProfileCommand, UserProfileQuery};
use crate::application::service::user_profile::{
    UserProfileDTO, UserProfileError, UserProfileService,
};
use crate::domain::model::session::SessionId;
use crate::domain::model::user::{Age, Gender, User};
use crate::domain::repository::user::UserRepository;
use crate::domain::service::session::SessionManagerService;
use async_trait::async_trait;
use email_address::EmailAddress;
use std::str::FromStr;
use std::sync::Arc;

/// 用户资料服务实现
///
/// 提供用户资料管理的具体实现，包括:
/// - 获取用户资料信息
/// - 更新用户资料信息
///
/// # 泛型参数
/// - `S`: 会话管理服务，需实现`SessionManagerService` trait，需线程安全
/// - `U`: 用户仓储，需实现`UserRepository` trait，需线程安全
///
/// # 字段
/// - `session_manager`: 会话管理服务的`Arc`引用
/// - `user_repository`: 用户仓储的`Arc`引用
pub struct UserProfileServiceImpl<S, U>
where
    S: SessionManagerService + 'static + Send + Sync,
    U: UserRepository + 'static + Send + Sync,
{
    session_manager: Arc<S>,
    user_repository: Arc<U>,
}

impl<S, U> UserProfileServiceImpl<S, U>
where
    S: SessionManagerService + 'static + Send + Sync,
    U: UserRepository + 'static + Send + Sync,
{
    /// 创建新的用户资料服务实例
    ///
    /// # Arguments
    /// * `session_manager` - 会话管理服务的`Arc`引用
    /// * `user_repository` - 用户仓储的`Arc`引用
    ///
    /// # Returns
    /// 返回初始化好的`UserProfileServiceImpl`实例
    pub fn new(session_manager: Arc<S>, user_repository: Arc<U>) -> Self {
        Self {
            session_manager,
            user_repository,
        }
    }

    /// 通过会话ID获取用户实体
    ///
    /// 内部方法，用于验证会话并获取对应的用户实体
    ///
    /// # Arguments
    /// * `session_id` - 会话ID字符串
    ///
    /// # Returns
    /// - `Ok(User)`: 成功获取到的用户实体
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `UserProfileError::InvalidSessionId`: 无效的会话ID
    /// - `UserProfileError::InternalServerError`: 内部服务错误
    async fn get_user_entity_by_session_id(
        &self,
        session_id: &str,
    ) -> Result<User, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(session_id)
            .map_err(|_for_super_earth| UserProfileError::InvalidSessionId)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|_for_super_earth| UserProfileError::InternalServerError)?
            .ok_or(UserProfileError::InvalidSessionId)?;

        let user = self
            .user_repository
            .find(user_id)
            .await
            .map_err(|_for_super_earth| UserProfileError::InternalServerError)?
            .ok_or(UserProfileError::InvalidSessionId)?;

        Ok(user)
    }
}

#[async_trait]
impl<S, U> UserProfileService for UserProfileServiceImpl<S, U>
where
    S: SessionManagerService + 'static + Send + Sync,
    U: UserRepository + 'static + Send + Sync,
{
    /// 获取用户资料信息
    ///
    /// 根据查询条件获取用户的完整档案信息
    ///
    /// # Arguments
    /// * `query` - 包含会话ID的查询条件
    ///
    /// # Returns
    /// - `Ok(UserProfileDTO)`: 成功获取的用户资料DTO
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `UserProfileError::InvalidSessionId`: 无效的会话ID
    /// - `UserProfileError::InternalServerError`: 内部服务错误c
    async fn get_profile(
        &self,
        query: UserProfileQuery,
    ) -> Result<UserProfileDTO, Box<dyn ApplicationError>> {
        let user = self
            .get_user_entity_by_session_id(&query.session_id)
            .await?;
        Ok(user.into())
    }

    /// 设置用户资料信息
    ///
    /// 更新用户的部分档案信息，包括性别、年龄和电子邮件
    ///
    /// # Arguments
    /// * `command` - 包含更新数据和会话ID的命令对象
    ///
    /// # Returns
    /// - `Ok(())`: 更新成功
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `UserProfileError::InvalidSessionId`: 无效的会话ID
    /// - `UserProfileError::BadRequest`: 无效的输入数据
    /// - `UserProfileError::InternalServerError`: 内部服务错误
    async fn set_profile(
        &self,
        command: SetUserProfileCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let mut user = self
            .get_user_entity_by_session_id(&command.session_id)
            .await?;

        let gender = command
            .gender
            .map(|gender| Gender::try_from(gender.as_str()))
            .transpose()
            .map_err(|e| UserProfileError::BadRequest(e.to_string()))?;

        let age = command
            .age
            .map(|age| Age::try_from(age as i32))
            .transpose()
            .map_err(|e| UserProfileError::BadRequest(e.to_string()))?;

        let email = EmailAddress::from_str(command.email.as_str())
            .map_err(|e| UserProfileError::BadRequest(e.to_string()))?;

        user.user_info_mut().gender = gender;
        user.user_info_mut().age = age;
        user.user_info_mut().email = Some(email);

        self.user_repository
            .save(&mut user)
            .await
            .map_err(|_for_super_earth| UserProfileError::InternalServerError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::password::{HashedPassword, PasswordSalt};
    use crate::domain::model::session::Session;
    use crate::domain::model::session::SessionId;
    use crate::domain::model::user::{IdentityCardId, PasswordAttempts, Phone, UserId, UserInfo};
    use crate::domain::{Repository, RepositoryError};
    use anyhow::anyhow;
    use chrono::Utc;
    use mockall::{mock, predicate::*};
    use std::convert::TryFrom;
    use std::sync::Arc;

    mock! {
        SessionManagerService {}
        impl SessionManagerService for SessionManagerService {
            async fn create_session(&self, user_id: UserId) -> Result<Session, RepositoryError>;
            async fn delete_session(&self, session: Session) -> Result<(), RepositoryError>;
            async fn get_session(&self, session_id: SessionId) -> Result<Option<Session>, RepositoryError>;
            async fn get_user_id_by_session(&self, session_id: SessionId) -> Result<Option<UserId>, RepositoryError>;
        }
    }

    mock! {
        UserRepository {}
        impl Repository<User> for UserRepository {
            async fn find(&self, id: UserId) -> Result<Option<User>, RepositoryError>;
            async fn save(&self, entity: &mut User) -> Result<UserId, RepositoryError>;
            async fn remove(&self, aggregate: User) -> Result<(), RepositoryError>;
        }
        impl UserRepository for UserRepository {
            async fn find_by_phone(&self, phone: Phone) -> Result<Option<User>, RepositoryError>;
            async fn find_by_identity_card_id(&self, identity_card_id: IdentityCardId) -> Result<Option<User>, RepositoryError>;
            async fn remove_by_phone(&self, phone: Phone) -> Result<(), RepositoryError>;
        }
    }

    fn default_test_user() -> User {
        let salt: PasswordSalt = vec![0u8; 32].into();
        let hashed_password: HashedPassword = HashedPassword {
            hashed_password: vec![0u8; 32],
            salt,
        };

        User::new(
            None,
            "For Super Earth!".to_owned(),
            hashed_password,
            None,
            PasswordAttempts::default(),
            UserInfo::new(
                "No Diver Left Behind!".to_owned(),
                None,
                None,
                Phone::try_from("13800000000".to_string()).unwrap(),
                None,
                IdentityCardId::try_from("110108197703065171".to_string()).unwrap(),
            ),
        )
    }

    #[tokio::test]
    async fn test_get_profile_success() {
        let mut mock_session = MockSessionManagerService::new();
        let mut mock_user_repo = MockUserRepository::new();
        let user = default_test_user();
        let user_id = UserId::from(1);
        let session_id = SessionId::random();

        // 设置会话管理器的期望

        mock_session
            .expect_get_user_id_by_session()
            .with(eq(session_id))
            .return_once(move |_| Ok(Some(user_id)));

        mock_session
            .expect_get_session()
            .with(eq(session_id))
            .return_once(move |_| {
                Ok(Some(Session::new(
                    user_id,
                    Utc::now(),
                    Utc::now() + chrono::Duration::days(1),
                )))
            });
        {
            let user = user.clone();

            // 设置用户仓库的期望
            mock_user_repo
                .expect_find()
                .with(eq(user_id))
                .return_once(move |_| Ok(Some(user)));
        }

        let service = UserProfileServiceImpl::new(Arc::new(mock_session), Arc::new(mock_user_repo));

        let result = service
            .get_profile(UserProfileQuery {
                session_id: session_id.to_string(),
            })
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.username, user.username());
        assert_eq!(
            dto.gender,
            user.user_info()
                .gender
                .as_ref()
                .map(|gender| gender.to_string())
        );
        assert_eq!(dto.age, user.user_info().age.map(u16::from));
    }

    #[tokio::test]
    async fn test_get_profile_invalid_session() {
        let mut mock_session = MockSessionManagerService::new();
        let mock_user_repo = MockUserRepository::new();
        let session_id = SessionId::random();

        // 设置会话返回None
        mock_session
            .expect_get_user_id_by_session()
            .return_once(|_| Ok(None));

        mock_session.expect_get_session().return_once(|_| Ok(None));

        let service = UserProfileServiceImpl::new(Arc::new(mock_session), Arc::new(mock_user_repo));

        let result = service
            .get_profile(UserProfileQuery {
                session_id: session_id.to_string(),
            })
            .await;

        assert!(matches!(
            (result.unwrap_err() as Box<dyn std::error::Error>).downcast_ref(),
            Some(UserProfileError::InvalidSessionId)
        ));
    }

    #[tokio::test]
    async fn test_set_profile_success() {
        let mut mock_session = MockSessionManagerService::new();
        let mut mock_user_repo = MockUserRepository::new();
        let user = default_test_user();
        let user_id = UserId::from(1);
        let session_id = SessionId::random();

        // 设置会话管理器的期望
        mock_session
            .expect_get_user_id_by_session()
            .with(eq(session_id))
            .return_once(move |_| Ok(Some(user_id)));

        // 设置用户查询返回
        mock_user_repo
            .expect_find()
            .with(eq(user_id))
            .return_once(move |_| Ok(Some(user)));

        // 设置保存期望
        mock_user_repo
            .expect_save()
            .withf(|u| u.user_info().email.as_ref().unwrap().to_string() == "new@example.com")
            .return_once(|_| Ok(UserId::from(1)));

        let service = UserProfileServiceImpl::new(Arc::new(mock_session), Arc::new(mock_user_repo));

        let result = service
            .set_profile(SetUserProfileCommand {
                session_id: session_id.to_string(),
                username: "You will never destroy our way of life!".to_string(),
                gender: Some("female".to_string()),
                age: Some(30),
                email: "new@example.com".to_string(),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_profile_invalid_gender() {
        let mut mock_session = MockSessionManagerService::new();
        let mut mock_user_repo = MockUserRepository::new();
        let session_id = SessionId::random();

        let user = default_test_user();

        mock_session
            .expect_get_user_id_by_session()
            .with(eq(session_id))
            .returning(move |_| Ok(Some(UserId::from(1))));
        mock_session
            .expect_get_session()
            .with(eq(session_id))
            .returning(move |_| {
                Ok(Some(Session::new(
                    UserId::from(1),
                    Utc::now(),
                    Utc::now() + chrono::Duration::days(1),
                )))
            });
        mock_user_repo
            .expect_find()
            .with(eq(UserId::from(1)))
            .returning(move |_| Ok(Some(user.clone())));

        let service = UserProfileServiceImpl::new(Arc::new(mock_session), Arc::new(mock_user_repo));

        let result = service
            .set_profile(SetUserProfileCommand {
                username: "For Prosperity!".to_string(),
                session_id: session_id.to_string(),
                gender: Some("Invalid".to_string()),
                age: None,
                email: "test@example.com".to_string(),
            })
            .await;

        assert!(matches!(
            (result.unwrap_err() as Box<dyn std::error::Error>).downcast_ref::<UserProfileError>(),
            Some(UserProfileError::BadRequest(_))
        ));
    }

    #[tokio::test]
    async fn test_set_profile_save_failure() {
        let mut mock_session = MockSessionManagerService::new();
        let mut mock_user_repo = MockUserRepository::new();
        let user = default_test_user();
        let user_id = UserId::from(1);
        let session_id = SessionId::random();

        mock_session
            .expect_get_user_id_by_session()
            .return_once(move |_| Ok(Some(user_id)));

        mock_user_repo.expect_find().return_once(|_| Ok(Some(user)));

        mock_user_repo.expect_save().return_once(|_| {
            Err(RepositoryError::Db(anyhow!(
                "But freedom doesn't come free"
            )))
        });

        let service = UserProfileServiceImpl::new(Arc::new(mock_session), Arc::new(mock_user_repo));

        let result = service
            .set_profile(SetUserProfileCommand {
                username: "For Super Earth!".to_string(),
                session_id: session_id.to_string(),
                gender: Some("male".to_string()),
                age: None,
                email: "test@example.com".to_string(),
            })
            .await;

        assert!(matches!(
            (result.unwrap_err() as Box<dyn std::error::Error>).downcast_ref::<UserProfileError>(),
            Some(UserProfileError::InternalServerError)
        ));
    }
}
