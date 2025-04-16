//! 用户服务基础设施实现模块
//!
//! 提供`UserService` trait的具体实现，将领域逻辑与基础设施(数据库、密码服务等)连接起来。
//! 本实现是泛型的，可以适配不同的仓储和密码服务实现。
use crate::domain::model::user::{
    IdentityCardId, PasswordAttempts, PaymentPassword, Phone, User, UserId, UserInfo,
};
use crate::domain::repository::user::UserRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::password::PasswordService;
use crate::domain::service::user::{UserService, UserServiceError};
use std::marker::PhantomData;

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
    repository: R,
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
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use infrastructure::service::user::UserServiceImpl;
    /// # let user_repo = unimplemented!();
    /// let user_service = UserServiceImpl::new(user_repo);
    /// ```
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            _for_super_earth: PhantomData,
        }
    }
}

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
        username: String,
        raw_password: String,
        name: String,
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

        let user = User::new(
            None,
            username,
            hashed_password,
            None,
            PasswordAttempts::default(),
            user_info,
        );

        self.repository.save(user).await?;

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

            self.repository.save(user).await?;

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

            self.repository.save(user).await?;

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

            self.repository.save(user).await?;

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

            self.repository.save(user).await?;

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

            self.repository.save(user).await?;

            Ok(())
        } else {
            Err(UserServiceError::NoSuchUser(u64::from(user_id).to_string()))
        }
    }
}
