//! 用户领域服务模块
//!
//! 提供与用户核心业务逻辑相关的服务接口定义和错误类型。
//! 这些服务操作涉及用户的完整生命周期管理，包括注册、删除、密码管理等功能。

use crate::domain::RepositoryError;
use crate::domain::model::user::{
    IdentityCardId, PasswordAttempts, PasswordError, PaymentPassword, Phone, RawPassword, RealName,
    User, UserId, UserInfo, Username,
};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use thiserror::Error;

/// 用户服务操作可能产生的错误类型
///
/// 包含了基础设施错误、业务规则违反等各种错误情况
#[derive(Error, Debug)]
pub enum UserServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),

    /// 用户已存在（手机号或身份证号重复）
    #[error("user with same phone number or identity card id exists: {0} {1}")]
    UserExists(String, String),

    /// 找不到指定用户
    #[error("no user with specified phone number: {0}")]
    NoSuchUser(String),

    /// 支付密码尝试次数超限
    #[error("payment password max attempts exceeded: {0}")]
    PaymentPasswordMaxAttemptsExceed(u8),

    /// 无效密码
    #[error("invalid password")]
    InvalidPassword,
}

impl From<RepositoryError> for UserServiceError {
    fn from(value: RepositoryError) -> Self {
        UserServiceError::InfrastructureError(value.into())
    }
}

impl From<PasswordError> for UserServiceError {
    fn from(value: PasswordError) -> Self {
        match value {
            PasswordError::InvalidPassword => UserServiceError::InvalidPassword,
            PasswordError::MaxAttemptsExceeded(attempts) => {
                UserServiceError::PaymentPasswordMaxAttemptsExceed(attempts)
            }
        }
    }
}

/// 用户领域服务接口
///
/// 定义了对用户实体进行业务操作的核心契约。
/// 所有方法都是异步的，返回实现了`Future` trait的结果。
#[async_trait]
pub trait UserService: 'static + Sync + Send {
    /// 注册新用户
    ///
    /// # Arguments
    /// * `username` - 用户名
    /// * `raw_password` - 明文密码（将由服务进行哈希处理）
    /// * `name` - 用户真实姓名
    /// * `phone` - 用户手机号
    /// * `identity_card_id` - 用户身份证号
    ///
    /// # Returns
    /// * `Ok(())` - 注册成功
    /// * `Err(UserServiceError)` - 注册失败及原因
    ///
    /// # Errors
    /// * `UserExists` - 手机号或身份证号已存在
    /// * `InfrastructureError` - 基础设施错误（如数据库访问失败）
    /// * `InvalidPassword` - 密码错误
    async fn register(
        &self,
        username: Username,
        raw_password: RawPassword,
        name: RealName,
        phone: Phone,
        identity_card_id: IdentityCardId,
    ) -> Result<(), UserServiceError>;

    /// 删除用户
    ///
    /// # Notes
    ///
    /// 此操作会级联删除用户**所有关联数据**
    ///
    /// # Arguments
    /// * `phone` - 要删除的用户手机号
    ///
    /// # Errors
    /// * `NoSuchUser` - 指定手机号的用户不存在
    /// * `InfrastructureError` - 基础设施错误
    async fn delete(&self, phone: Phone) -> Result<(), UserServiceError>;

    /// 验证用户登录密码
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
    ) -> Result<(), UserServiceError>;

    /// 验证用户支付密码
    ///
    /// # Arguments
    /// * `user` - 用户实体
    /// * `raw_password` - 用户提供的明文支付密码
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施或密码服务错误
    async fn verify_payment_password(
        &self,
        user: &User,
        raw_payment_password: String,
    ) -> Result<(), UserServiceError>;

    /// 设置用户登录密码
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `raw_password` - 新的明文密码
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施或密码服务错误
    async fn set_password(
        &self,
        user_id: UserId,
        raw_password: String,
    ) -> Result<(), UserServiceError>;

    /// 设置或清除支付密码
    ///
    /// # Notes
    ///
    /// 此操作不会重置支付密码错误尝试次数，需要调用者手动处理
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `payment_password` - 支付密码（None表示清除）
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施或密码服务错误
    async fn set_payment_password(
        &self,
        user_id: UserId,
        payment_password: Option<PaymentPassword>,
    ) -> Result<(), UserServiceError>;

    /// 直接设置支付密码错误尝试次数
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `password_attempts` - 新的尝试次数值
    ///
    /// # Errors
    /// * `NoSuchUser` - 用户不存在
    /// * `InfrastructureError` - 基础设施错误
    async fn set_wrong_payment_password_tried(
        &self,
        user_id: UserId,
        password_attempts: PasswordAttempts,
    ) -> Result<(), UserServiceError>;

    /// 重置支付密码错误尝试次数为0
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
    ) -> Result<(), UserServiceError>;

    /// 递增支付密码错误尝试次数
    ///
    /// 当用户输入错误支付密码时调用
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
    ) -> Result<(), UserServiceError>;

    /// 更新用户详细信息
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
    ) -> Result<(), UserServiceError>;
}
