//! 用户服务基础设施实现模块
//!
//! 提供`UserService` trait的具体实现，将领域逻辑与基础设施(数据库、密码服务等)连接起来。
//! 本实现是泛型的，可以适配不同的仓储和密码服务实现。
use crate::domain::model::user::{
    IdentityCardId, PasswordAttempts, PaymentPassword, Phone, RawPassword, RealName, User, UserId,
    UserInfo, Username,
};
use crate::domain::repository::user::UserRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::password::PasswordService;
use crate::domain::service::user::{UserService, UserServiceError};
use async_trait::async_trait;
use std::marker::PhantomData;
use std::sync::Arc;

/// 用户服务具体实现
///
/// 泛型参数：
/// - `R`: 用户仓储实现
/// - `P`: 密码服务实现
///
/// # 类型约束
/// - `R`必须实现`UserRepository` trait
/// - `P`必须实现`PasswordService` trait
pub struct UserServiceImpl<R, P>
where
    R: UserRepository,
    P: PasswordService,
{
    /// 用户仓储实例
    repository: Arc<R>,
    /// 用于标记密码服务类型的PhantomData
    _for_super_earth: PhantomData<P>,
}

impl<R, P> UserServiceImpl<R, P>
where
    R: UserRepository,
    P: PasswordService,
{
    /// 创建新的用户服务实例
    ///
    /// # Arguments
    /// * `repository` - 用户仓储实现
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            _for_super_earth: PhantomData,
        }
    }
}

#[async_trait]
impl<R, P> UserService for UserServiceImpl<R, P>
where
    R: UserRepository + 'static + Send + Sync,
    P: PasswordService + 'static + Send + Sync,
{
    /// 用户注册实现
    ///
    /// # Notes
    ///
    /// 由于数据库表结构限制，登录密码和支付密码必须使用相同的盐值
    ///
    /// # Arguments
    /// * `username` - 用户名
    /// * `raw_password` - 明文密码
    /// * `name` - 真实姓名
    /// * `phone` - 手机号
    /// * `identity_card_id` - 身份证号
    ///
    /// # Errors
    /// * `UserExists` - 手机号或身份证号已注册
    /// * `InfrastructureError` - 密码哈希失败或存储错误
    async fn register(
        &self,
        username: Username,
        raw_password: RawPassword,
        name: RealName,
        phone: Phone,
        identity_card_id: IdentityCardId,
    ) -> Result<(), UserServiceError> {
        if self
            .repository
            .find_by_phone(phone.clone())
            .await?
            .is_some()
        {
            return Err(UserServiceError::UserExists(phone.into(), "".into()));
        }

        if self
            .repository
            .find_by_identity_card_id(identity_card_id.clone())
            .await?
            .is_some()
        {
            return Err(UserServiceError::UserExists(
                "".into(),
                identity_card_id.into(),
            ));
        }

        let salt = P::generate_salt();

        let user_info = UserInfo::new(name, None, None, phone, None, identity_card_id);

        let hashed_password = P::hash_password(raw_password.as_bytes(), salt.clone()).unwrap();

        let mut user = User::new(
            None,
            username,
            hashed_password,
            None,
            PasswordAttempts::default(),
            user_info,
        );

        self.repository.save(&mut user).await?;

        Ok(())
    }

    /// 删除用户实现
    ///
    /// # Notes
    ///
    /// 此操作会级联删除用户**所有关联数据**
    ///
    /// # Arguments
    /// * `phone` - 用户手机号
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施错误
    async fn delete(&self, phone: Phone) -> Result<(), UserServiceError> {
        self.repository.remove_by_phone(phone).await?;

        Ok(())
    }

    /// 验证用户登录密码实现
    ///
    /// # Arguments
    /// * `user` - 用户实体
    /// * `raw_password` - 用户提供的明文密码
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施或密码服务错误
    async fn verify_password(
        &self,
        user: &User,
        raw_password: String,
    ) -> Result<(), UserServiceError> {
        let pass =
            P::verify(raw_password.as_bytes(), user.hashed_password().clone()).map_err(|e| {
                UserServiceError::InfrastructureError(ServiceError::RelatedServiceError(e))
            })?;

        if !pass {
            Err(UserServiceError::InvalidPassword)
        } else {
            Ok(())
        }
    }

    /// 设置登录密码实现
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `raw_password` - 新明文密码
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 密码哈希失败或基础设施错误
    async fn set_password(
        &self,
        user_id: UserId,
        raw_password: String,
    ) -> Result<(), UserServiceError> {
        if let Some(mut user) = self.repository.find(user_id).await? {
            let new_password =
                P::hash_password(raw_password.as_bytes(), user.hashed_password().salt.clone())
                    .map_err(|e| {
                        UserServiceError::InfrastructureError(ServiceError::RelatedServiceError(e))
                    })?;

            user.set_hashed_password(new_password);

            self.repository.save(&mut user).await?;

            Ok(())
        } else {
            Err(UserServiceError::NoSuchUser(u64::from(user_id).to_string()))
        }
    }

    /// 设置支付密码实现
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `payment_password` - 支付密码(传入None表示清除)
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 密码哈希失败或基础设施错误
    async fn set_payment_password(
        &self,
        user_id: UserId,
        payment_password: Option<PaymentPassword>,
    ) -> Result<(), UserServiceError> {
        if let Some(mut user) = self.repository.find(user_id).await? {
            let new_password = payment_password
                .map(|p| {
                    P::hash_password(
                        String::from(p).as_bytes(),
                        user.hashed_password().salt.clone(),
                    )
                })
                .transpose()
                .map_err(|e| {
                    UserServiceError::InfrastructureError(ServiceError::RelatedServiceError(e))
                })?;

            user.set_hashed_payment_password(new_password);

            self.repository.save(&mut user).await?;

            Ok(())
        } else {
            Err(UserServiceError::NoSuchUser(u64::from(user_id).to_string()))
        }
    }

    /// 设置支付密码错误尝试次数实现
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `password_attempts` - 新的尝试次数
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施错误
    async fn set_wrong_payment_password_tried(
        &self,
        user_id: UserId,
        password_attempts: PasswordAttempts,
    ) -> Result<(), UserServiceError> {
        if let Some(mut user) = self.repository.find(user_id).await? {
            *user.wrong_payment_password_tried_mut() = password_attempts;

            self.repository.save(&mut user).await?;

            Ok(())
        } else {
            Err(UserServiceError::NoSuchUser(u64::from(user_id).to_string()))
        }
    }

    /// 重置支付密码错误尝试次数实现
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施错误
    async fn clear_wrong_payment_password_tried(
        &self,
        user_id: UserId,
    ) -> Result<(), UserServiceError> {
        self.set_wrong_payment_password_tried(user_id, PasswordAttempts::default())
            .await
    }

    /// 递增支付密码错误尝试次数实现
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `PaymentPasswordMaxAttemptsExceed` - 尝试次数已达上限
    /// * `InfrastructureError` - 基础设施错误
    async fn increment_wrong_payment_password_tried(
        &self,
        user_id: UserId,
    ) -> Result<(), UserServiceError> {
        if let Some(mut user) = self.repository.find(user_id).await? {
            user.wrong_payment_password_tried_mut().increment()?;

            self.repository.save(&mut user).await?;

            Ok(())
        } else {
            Err(UserServiceError::NoSuchUser(u64::from(user_id).to_string()))
        }
    }

    /// 更新用户信息实现
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `user_info` - 新的用户信息
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施错误
    async fn set_user_info(
        &self,
        user_id: UserId,
        user_info: UserInfo,
    ) -> Result<(), UserServiceError> {
        if let Some(mut user) = self.repository.find(user_id).await? {
            user.set_user_info(user_info);

            self.repository.save(&mut user).await?;

            Ok(())
        } else {
            Err(UserServiceError::NoSuchUser(u64::from(user_id).to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::password::{HashedPassword, PasswordSalt};
    use crate::domain::{Identifiable, Repository, RepositoryError};
    use crate::infrastructure::service::password::{
        Argon2PasswordServiceImpl, MockPasswordServiceImpl,
    };
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        UserRepo {}

        #[async_trait]
        impl Repository<User> for UserRepo {
            async fn find(&self, id: UserId) -> Result<Option<User>, RepositoryError>;
            async fn save(&self, user: &mut User) -> Result<UserId, RepositoryError>;
            async fn remove(&self, aggregate: User) -> Result<(), RepositoryError>;
        }

        #[async_trait]
        impl UserRepository for UserRepo {
            async fn find_by_phone(&self,phone: Phone,) -> Result<Option<User>, RepositoryError>;

            async fn find_by_identity_card_id(&self,identity_card_id: IdentityCardId,) -> Result<Option<User>, RepositoryError>;

            async fn remove_by_phone(&self,phone: Phone,) -> Result<(), RepositoryError>;
        }
    }

    type MockRepo = MockUserRepo;

    fn default_test_user() -> User {
        let salt: PasswordSalt = vec![0u8; 32].into();
        let hashed_password: HashedPassword = HashedPassword {
            hashed_password: vec![0u8; 32],
            salt,
        };

        User::new(
            None,
            Username::try_from("For Super Earth!".to_owned()).unwrap(),
            hashed_password,
            None,
            PasswordAttempts::default(),
            UserInfo::new(
                RealName::try_from("DemoHasLanded".to_owned()).unwrap(),
                None,
                None,
                Phone::try_from("13800000000".to_string()).unwrap(),
                None,
                IdentityCardId::try_from("110108197703065171".to_string()).unwrap(),
            ),
        )
    }

    // 注册测试
    #[tokio::test]
    async fn register_success() {
        let username = "For Super Earth!";
        let raw_password = "";
        let name = "DemoHasLanded";
        let phone = "13800000000";
        let identity_card_id = "110108197703065171";

        let mut repo = MockRepo::new();

        repo.expect_find().returning(|_| Ok(None));
        repo.expect_find_by_phone().returning(|_| Ok(None));
        repo.expect_find_by_identity_card_id()
            .returning(|_| Ok(None));

        repo.expect_save()
            .withf(move |user| {
                user.username() == username
                    && &*user.user_info().name == name
                    && &*user.user_info().phone == phone
                    && &*user.user_info().identity_card_id == identity_card_id
            })
            .times(1)
            .returning(|_| Ok(UserId::from(1)));

        let service = UserServiceImpl::<_, Argon2PasswordServiceImpl>::new(Arc::new(repo));

        let result = service
            .register(
                Username::try_from(username.to_string()).unwrap(),
                RawPassword::try_from(raw_password.to_string()).unwrap(),
                RealName::try_from(name.to_string()).unwrap(),
                Phone::try_from(phone.to_string()).unwrap(),
                IdentityCardId::try_from(identity_card_id.to_string()).unwrap(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn register_phone_exists() {
        let mut repo = MockRepo::new();

        repo.expect_find().returning(|_| Ok(None));
        repo.expect_find_by_phone()
            .returning(|_| Ok(Some(default_test_user())));
        repo.expect_find_by_identity_card_id()
            .returning(|_| Ok(None));

        let service = UserServiceImpl::<_, Argon2PasswordServiceImpl>::new(Arc::new(repo));

        let default_user = default_test_user();

        let result = service
            .register(
                Username::try_from(default_user.username().to_owned()).unwrap(),
                RawPassword::try_from("".to_owned()).unwrap(),
                default_user.user_info().name.to_owned(),
                default_user.user_info().phone.clone(),
                default_user.user_info().identity_card_id.clone(),
            )
            .await;

        assert!(matches!(result, Err(UserServiceError::UserExists(_, _))));
    }

    // 删除用户测试
    #[tokio::test]
    async fn delete_success() {
        let mut repo = MockUserRepo::new();
        repo.expect_remove_by_phone()
            .withf(|x| {
                let default_user = default_test_user();

                let actual: &str = x.as_ref();
                let expected: &str = default_user.user_info().phone.as_ref();

                expected == actual
            })
            .times(1)
            .returning(|_| Ok(()));

        let service = UserServiceImpl::<_, Argon2PasswordServiceImpl>::new(Arc::new(repo));

        let result = service
            .delete(default_test_user().user_info().phone.to_owned())
            .await;
        assert!(result.is_ok());
    }

    // 设置密码测试
    #[tokio::test]
    async fn set_password_success() {
        let user = default_test_user();

        let mut repo = MockUserRepo::new();

        let prev_hashed_password = user.hashed_password().clone();
        {
            let user = user.clone();
            repo.expect_find()
                .returning(move |_| Ok(Some(user.clone())));
        }

        repo.expect_save()
            .withf(move |u| {
                let current_hashed_password = u.hashed_password().clone();
                current_hashed_password.hashed_password != prev_hashed_password.hashed_password
                    && current_hashed_password.salt == prev_hashed_password.salt
            })
            .times(1)
            .returning(|_| Ok(UserId::from(1)));

        let service = UserServiceImpl::<_, MockPasswordServiceImpl>::new(Arc::new(repo));

        let result = service
            .set_password(UserId::from(1), "new_password".to_string())
            .await;
        assert!(result.is_ok());
    }

    // 支付密码错误尝试测试
    #[tokio::test]
    async fn increment_payment_attempts_exceed() {
        let mut user = default_test_user();

        user.set_id(UserId::from(1));

        let user_id = user.get_id().unwrap();

        *user.wrong_payment_password_tried_mut() =
            PasswordAttempts::try_from(PasswordAttempts::MAX).unwrap();

        let mut repo = MockUserRepo::new();
        repo.expect_find()
            .returning(move |_| Ok(Some(user.clone())));

        let service = UserServiceImpl::<_, Argon2PasswordServiceImpl>::new(Arc::new(repo));

        let result = service
            .increment_wrong_payment_password_tried(user_id)
            .await;
        assert!(matches!(
            result,
            Err(UserServiceError::PaymentPasswordMaxAttemptsExceed(_))
        ));
    }
}
