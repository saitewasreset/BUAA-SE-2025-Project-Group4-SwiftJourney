//! 用户管理服务实现模块
//!
//! 本模块提供了`UserManagerService`接口的具体实现。
//!
//! ## 主要组件
//! - `UserManagerServiceImpl`: 用户管理服务的具体实现
//! - 依赖三个核心组件:
//!   - 用户服务(`UserService`)
//!   - 用户仓储(`UserRepository`)
//!   - 会话管理服务(`SessionManagerService`)
use crate::application::commands::user_manager::{
    UserLoginCommand, UserLogoutCommand, UserRegisterCommand, UserUpdatePasswordCommand,
};
use crate::application::service::user_manager::{UserManagerError, UserManagerService};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::session::SessionId;
use crate::domain::model::user::{IdentityCardId, Phone, RawPassword, RealName, Username};
use crate::domain::repository::user::UserRepository;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::user::UserService;
use crate::domain::{DbId, Identifiable};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::error;

/// 用户管理服务实现
///
/// 实现了`UserManagerService`接口，协调用户服务、用户仓储和会话管理服务
/// 来完成用户注册、登录和登出操作。
///
/// # 类型参数
/// - `U`: 用户服务类型，需实现`UserService` trait
/// - `R`: 用户仓储类型，需实现`UserRepository` trait
/// - `S`: 会话管理服务类型，需实现`SessionManagerService` trait
///
/// # 字段
/// - `user_service`: 用户领域服务
/// - `user_repository`: 用户仓储
/// - `session_manager`: 会话管理服务
pub struct UserManagerServiceImpl<U, R, S>
where
    U: UserService + 'static + Send + Sync,
    R: UserRepository + 'static + Send + Sync,
    S: SessionManagerService + 'static + Send + Sync,
{
    user_service: Arc<U>,
    user_repository: Arc<R>,
    session_manager: Arc<S>,
}

impl<U, R, S> UserManagerServiceImpl<U, R, S>
where
    U: UserService + 'static + Send + Sync,
    R: UserRepository + 'static + Send + Sync,
    S: SessionManagerService + 'static + Send + Sync,
{
    /// 创建新的用户管理服务实例
    ///
    /// # Arguments
    /// * `user_service`: 用户领域服务
    /// * `user_repository`: 用户仓储
    /// * `session_manager`: 会话管理服务
    ///
    /// # Returns
    /// 返回新的`UserManagerServiceImpl`实例
    pub fn new(user_service: Arc<U>, user_repository: Arc<R>, session_manager: Arc<S>) -> Self {
        UserManagerServiceImpl {
            user_service,
            user_repository,
            session_manager,
        }
    }
}

#[async_trait]
impl<U, R, S> UserManagerService for UserManagerServiceImpl<U, R, S>
where
    U: UserService + 'static + Send + Sync,
    R: UserRepository + 'static + Send + Sync,
    S: SessionManagerService + 'static + Send + Sync,
{
    /// 处理用户注册请求
    ///
    /// 1. 验证并转换输入数据为领域对象
    /// 2. 调用用户服务完成注册
    ///
    /// # Arguments
    /// * `command`: 用户注册命令
    ///
    /// # Errors
    /// 可能返回以下错误:
    /// - `UserManagerError::InvalidUsernameFormat`: 无效用户名格式
    /// - `UserManagerError::InvalidPasswordFormat`: 无效密码格式
    /// - `UserManagerError::InvalidNameFormat`: 无效姓名格式
    /// - `GeneralError::BadRequest`: 无效的手机号或身份证号格式
    /// - `UserManagerError::UserAlreadyExists`: 用户已存在
    /// - `GeneralError::InternalServerError`: 服务内部错误
    async fn register(
        &self,
        command: UserRegisterCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let phone =
            Phone::try_from(command.phone).map_err(|e| GeneralError::BadRequest(e.to_string()))?;
        let identity_card_id = IdentityCardId::try_from(command.identity_card_id)
            .map_err(|e| GeneralError::BadRequest(e.to_string()))?;

        let username = Username::try_from(command.username)
            .map_err(|_for_super_earth| UserManagerError::InvalidUsernameFormat)?;
        let raw_password = RawPassword::try_from(command.password)
            .map_err(|_for_super_earth| UserManagerError::InvalidPasswordFormat)?;

        let name = RealName::try_from(command.name)
            .map_err(|_for_super_earth| UserManagerError::InvalidNameFormat)?;

        self.user_service
            .register(username, raw_password, name, phone, identity_card_id)
            .await
            .map_err(|e| e.into())
    }

    /// 处理用户登录请求
    ///
    /// 1. 验证手机号格式
    /// 2. 通过仓储查找用户
    /// 3. 验证密码
    /// 4. 创建新会话
    ///
    /// # Arguments
    /// * `command`: 用户登录命令
    ///
    /// # Returns
    /// 成功时返回新创建的会话ID
    ///
    /// # Errors
    /// 可能返回以下错误:
    /// - `GeneralError::BadRequest`: 无效的手机号格式
    /// - `UserManagerError::InvalidPhoneNumberOrPassword`: 无效的手机号或密码
    /// - `GeneralError::InternalServerError`: 仓储或服务内部错误
    async fn login(
        &self,
        command: UserLoginCommand,
    ) -> Result<SessionId, Box<dyn ApplicationError>> {
        let phone =
            Phone::try_from(command.phone).map_err(|e| GeneralError::BadRequest(e.to_string()))?;

        let user = self
            .user_repository
            .find_by_phone(phone)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(UserManagerError::InvalidPhoneNumberOrPassword)?;

        self.user_service
            .verify_password(&user, command.password)
            .await?;

        let session = self
            .session_manager
            .create_session(user.get_id().unwrap())
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(session.session_id())
    }

    /// 处理用户登出请求
    ///
    /// 1. 验证会话ID格式
    /// 2. 查找并删除会话
    ///
    /// # Arguments
    /// * `command`: 用户登出命令
    ///
    /// # Errors
    /// 可能返回以下错误:
    /// - `GeneralError::InvalidSessionId`: 无效的会话ID
    /// - `GeneralError::InternalServerError`: 服务内部错误
    async fn logout(&self, command: UserLogoutCommand) -> Result<(), Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(command.session_id.as_str())
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;
        if let Some(session) = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
        {
            self.session_manager
                .delete_session(session)
                .await
                .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

            Ok(())
        } else {
            Err(GeneralError::InvalidSessionId.into())
        }
    }

    async fn update_password(
        &self,
        command: UserUpdatePasswordCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(command.session_id.as_str())
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;
        if let Some(user_id) = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
        {
            let user = self
                .user_repository
                .find(user_id)
                .await
                .map_err(|e| {
                    error!("failed load user {} from db: {}", user_id.to_db_value(), e);

                    GeneralError::InternalServerError
                })?
                .ok_or(GeneralError::InvalidSessionId)?;

            self.user_service
                .verify_password(&user, command.origin_password)
                .await?;

            self.user_service
                .set_password(user_id, command.new_password)
                .await?;

            Ok(())
        } else {
            Err(GeneralError::InvalidSessionId.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::password::{HashedPassword, PasswordSalt};
    use crate::domain::model::session::{Session, SessionId};
    use crate::domain::model::user::{
        IdentityCardId, PasswordAttempts, PaymentPassword, Phone, RealName, User, UserId, UserInfo,
        Username,
    };
    use crate::domain::service::user::UserServiceError;
    use crate::domain::{Identifiable, Repository, RepositoryError};
    use chrono::Utc;
    use mockall::{mock, predicate::*};
    use std::convert::TryFrom;
    use std::sync::Arc;

    // 模拟UserService
    mock! {
        UserService {}
        #[async_trait]
        impl UserService for UserService {
            async fn register(
                &self,
                username: Username,
                raw_password: RawPassword,
                name: RealName,
                phone: Phone,
                identity_card_id: IdentityCardId,
            ) -> Result<(), UserServiceError>;
            async fn delete(&self, phone: Phone) -> Result<(), UserServiceError>;
            async fn verify_password(
                &self,
                user: &User,
                raw_password: String,
            ) -> Result<(), UserServiceError>;

            async fn verify_payment_password(
                &self,
                user: &User,
                raw_payment_password: String,
            ) -> Result<(), UserServiceError>;

            async fn set_password(
                &self,
                user_id: UserId,
                raw_password: String,
            ) -> Result<(), UserServiceError>;
            async fn set_payment_password(
                &self,
                user_id: UserId,
                payment_password: Option<PaymentPassword>,
            ) -> Result<(), UserServiceError>;
            async fn set_wrong_payment_password_tried(
                &self,
                user_id: UserId,
                password_attempts: PasswordAttempts,
            ) -> Result<(), UserServiceError>;
            async fn clear_wrong_payment_password_tried(
                &self,
                user_id: UserId,
            ) -> Result<(), UserServiceError>;
            async fn increment_wrong_payment_password_tried(
                &self,
                user_id: UserId,
            ) -> Result<(), UserServiceError>;
            async fn set_user_info(
                &self,
                user_id: UserId,
                user_info: UserInfo,
            ) -> Result<(), UserServiceError>;
        }
    }

    // 模拟UserRepository
    mock! {
        UserRepo {}
        #[async_trait]
        impl Repository<User> for UserRepo {
            async fn find(&self, id: UserId) -> Result<Option<User>, RepositoryError>;
            async fn save(&self, entity: &mut User) -> Result<UserId, RepositoryError>;
            async fn remove(&self, aggregate: User) -> Result<(), RepositoryError>;
        }
        #[async_trait]
        impl UserRepository for UserRepo {
            async fn find_by_phone(&self, phone: Phone) -> Result<Option<User>, RepositoryError>;
            async fn find_by_identity_card_id(
                &self,
                identity_card_id: IdentityCardId,
            ) -> Result<Option<User>, RepositoryError>;
            async fn remove_by_phone(&self, phone: Phone) -> Result<(), RepositoryError>;
        }
    }

    // 模拟SessionManagerService
    mock! {
        SessionService {}
        #[async_trait]
        impl SessionManagerService for SessionService {
            async fn create_session(&self, user_id: UserId) -> Result<Session, RepositoryError>;
            async fn delete_session(&self, session: Session) -> Result<(), RepositoryError>;
            async fn get_session(&self, session_id: SessionId) -> Result<Option<Session>, RepositoryError>;
            async fn get_user_id_by_session(&self, session_id: SessionId) -> Result<Option<UserId>, RepositoryError>;
            async fn verify_session_id(&self, session_id_str: &str) -> Result<bool, RepositoryError>;
        }
    }

    // 创建测试用户
    fn create_test_user(id: Option<UserId>) -> User {
        let salt: PasswordSalt = vec![0u8; 32].into();
        let hashed_password: HashedPassword = HashedPassword {
            hashed_password: vec![0u8; 32],
            salt,
        };

        let mut user = User::new(
            id,
            Username::try_from("testuser".to_owned()).unwrap(),
            hashed_password,
            None,
            PasswordAttempts::default(),
            UserInfo::new(
                RealName::try_from("张三".to_owned()).unwrap(),
                None,
                None,
                Phone::try_from("13012345678".to_string()).unwrap(),
                None,
                IdentityCardId::try_from("11010519491231002X".to_string()).unwrap(),
            ),
        );

        if id.is_none() {
            user.set_id(UserId::from(1));
        }

        user
    }

    #[tokio::test]
    async fn test_register_success() {
        let mut mock_user_service = MockUserService::new();
        let mock_user_repo = MockUserRepo::new();
        let mock_session_service = MockSessionService::new();

        // 设置UserService的期望
        mock_user_service
            .expect_register()
            .with(
                eq(Username::try_from("testuser".to_string()).unwrap()),
                eq(RawPassword::try_from("password123".to_string()).unwrap()),
                eq(RealName::try_from("张三".to_string()).unwrap()),
                eq(Phone::try_from("13012345678".to_string()).unwrap()),
                eq(IdentityCardId::try_from("11010519491231002X".to_string()).unwrap()),
            )
            .return_once(|_, _, _, _, _| Ok(()));

        let service = UserManagerServiceImpl::new(
            Arc::new(mock_user_service),
            Arc::new(mock_user_repo),
            Arc::new(mock_session_service),
        );

        let result = service
            .register(UserRegisterCommand {
                phone: "13012345678".to_string(),
                username: "testuser".to_string(),
                password: "password123".to_string(),
                name: "张三".to_string(),
                identity_card_id: "11010519491231002X".to_string(),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login_success() {
        let mut mock_user_service = MockUserService::new();
        let mut mock_user_repo = MockUserRepo::new();
        let mut mock_session_service = MockSessionService::new();

        let user = create_test_user(Some(UserId::from(1)));
        {
            let user = user.clone();

            // 设置UserRepository的期望
            mock_user_repo
                .expect_find_by_phone()
                .with(eq(Phone::try_from("13012345678".to_string()).unwrap()))
                .return_once(|_| Ok(Some(user)));
        }

        {
            let user = user.clone();
            // 设置UserService的期望
            mock_user_service
                .expect_verify_password()
                .with(eq(user), eq("password123".to_string()))
                .return_once(|_, _| Ok(()));
        }

        // 设置SessionService的期望
        mock_session_service
            .expect_create_session()
            .with(eq(UserId::from(1)))
            .return_once(|_| {
                Ok(Session::new(
                    UserId::from(1),
                    Utc::now(),
                    Utc::now() + chrono::Duration::days(1),
                ))
            });

        let service = UserManagerServiceImpl::new(
            Arc::new(mock_user_service),
            Arc::new(mock_user_repo),
            Arc::new(mock_session_service),
        );

        let result = service
            .login(UserLoginCommand {
                phone: "13012345678".to_string(),
                password: "password123".to_string(),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logout_success() {
        let mock_user_service = MockUserService::new();
        let mock_user_repo = MockUserRepo::new();
        let mut mock_session_service = MockSessionService::new();

        let session_id = SessionId::random();
        let session = Session::new(
            UserId::from(1),
            Utc::now(),
            Utc::now() + chrono::Duration::days(1),
        );

        {
            let session = session.clone();
            // 设置SessionService的期望
            mock_session_service
                .expect_get_session()
                .with(eq(session_id))
                .return_once(|_| Ok(Some(session)));
        }

        {
            let session = session.clone();
            mock_session_service
                .expect_delete_session()
                .with(eq(session))
                .return_once(|_| Ok(()));
        }

        let service = UserManagerServiceImpl::new(
            Arc::new(mock_user_service),
            Arc::new(mock_user_repo),
            Arc::new(mock_session_service),
        );

        let result = service
            .logout(UserLogoutCommand {
                session_id: session_id.to_string(),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let mock_user_service = MockUserService::new();
        let mut mock_user_repo = MockUserRepo::new();
        let mock_session_service = MockSessionService::new();

        // 设置UserRepository的期望
        mock_user_repo
            .expect_find_by_phone()
            .with(eq(Phone::try_from("13012345678".to_string()).unwrap()))
            .return_once(|_| Ok(None));

        let service = UserManagerServiceImpl::new(
            Arc::new(mock_user_service),
            Arc::new(mock_user_repo),
            Arc::new(mock_session_service),
        );

        let result = service
            .login(UserLoginCommand {
                phone: "13012345678".to_string(),
                password: "password123".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error_message(),
            "Invalid phone number of password"
        );
    }

    #[tokio::test]
    async fn test_logout_invalid_session() {
        let mock_user_service = MockUserService::new();
        let mock_user_repo = MockUserRepo::new();
        let mut mock_session_service = MockSessionService::new();

        let session_id = SessionId::random();

        // 设置SessionService的期望
        mock_session_service
            .expect_get_session()
            .with(eq(session_id))
            .return_once(|_| Ok(None));

        let service = UserManagerServiceImpl::new(
            Arc::new(mock_user_service),
            Arc::new(mock_user_repo),
            Arc::new(mock_session_service),
        );

        let result = service
            .logout(UserLogoutCommand {
                session_id: session_id.to_string(),
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), 403);
    }
}
