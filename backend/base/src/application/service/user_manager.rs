//! 用户管理服务模块
//!
//! 本模块定义了用户管理相关的服务接口和数据结构，包括：
//! - 用户管理服务接口(`UserManagerService`)
//! - 用户管理相关的数据传输对象(DTO)
//! - 用户管理特定的错误类型
//!
//! ## 主要组件
//! - `UserManagerService`: 用户管理核心服务接口
//! - `UserRegisterDTO`: 用户注册数据传输对象
//! - `UserLoginDTO`: 用户登录数据传输对象
//! - `UserManagerError`: 用户管理特定错误类型

use crate::application::commands::user_manager::{
    UserLoginCommand, UserLogoutCommand, UserRegisterCommand, UserUpdatePasswordCommand,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::session::SessionId;
use crate::domain::service::user::UserServiceError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 用户注册数据传输对象(DTO)
///
/// 用于接收用户注册请求的数据结构，包含注册所需的所有字段。
/// 该结构体同时实现了`Serialize`和`Deserialize`，可直接用于API请求/响应。
///
/// # Fields
/// - `phone`: 用户手机号码
/// - `username`: 用户名
/// - `password`: 用户密码(明文)
/// - `name`: 用户真实姓名
/// - `identity_card_id`: 身份证号码(使用驼峰命名法序列化)
///
/// # Examples
///
/// ```json
/// {
///     "phone": "13012345678",
///     "username": "testuser",
///     "password": "password123",
///     "name": "张三",
///     "identityCardId": "11010519491231002X"
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserRegisterDTO {
    pub phone: String,
    pub username: String,
    pub password: String,
    pub name: String,
    #[serde(rename = "identityCardId")]
    pub identity_card_id: String,
}

/// 用户登录数据传输对象(DTO)
///
/// 用于接收用户登录请求的数据结构，包含登录凭证信息。
/// 该结构体同时实现了`Serialize`和`Deserialize`，可直接用于API请求/响应。
///
/// # Fields
/// - `phone`: 用户手机号码
/// - `password`: 用户密码(明文)
///
/// # Examples
///
/// ```json
/// {
///     "phone": "13012345678",
///     "password": "password123"
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLoginDTO {
    pub phone: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordDTO {
    pub origin_password: String,
    pub new_password: String,
}

/// 用户管理服务错误类型
///
/// 定义了用户管理服务可能返回的所有特定错误。
#[derive(Error, Debug)]
pub enum UserManagerError {
    /// 用户已存在错误(手机号已注册)
    #[error("User with phone {0} already exists")]
    UserAlreadyExists(String),
    /// 手机号或密码错误
    #[error("Invalid phone number of password")]
    InvalidPhoneNumberOrPassword,
    #[error("Invalid username")]
    /// 无效的用户名格式，详见RFC3
    InvalidUsernameFormat,
    #[error("Invalid password")]
    /// 无效的密码格式，详见RFC3
    InvalidPasswordFormat,
    #[error("Invalid name")]
    /// 无效的姓名格式，详见RFC3
    InvalidNameFormat,
}

impl ApplicationError for UserManagerError {
    fn error_code(&self) -> u32 {
        match self {
            UserManagerError::UserAlreadyExists(_) => 15001,
            UserManagerError::InvalidPhoneNumberOrPassword => 15002,
            UserManagerError::InvalidUsernameFormat => 15003,
            UserManagerError::InvalidPasswordFormat => 15004,
            UserManagerError::InvalidNameFormat => 15005,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

impl From<UserServiceError> for Box<dyn ApplicationError> {
    fn from(value: UserServiceError) -> Self {
        match value {
            UserServiceError::InfrastructureError(_) => GeneralError::InternalServerError.into(),
            UserServiceError::InvalidPassword => {
                UserManagerError::InvalidPhoneNumberOrPassword.into()
            }
            UserServiceError::NoSuchUser(_) => {
                UserManagerError::InvalidPhoneNumberOrPassword.into()
            }
            UserServiceError::UserExists(s, _) => UserManagerError::UserAlreadyExists(s).into(),
            UserServiceError::PaymentPasswordMaxAttemptsExceed(_) => todo!("应在交易模块实现"),
        }
    }
}

/// 用户管理服务接口
///
/// 定义了用户管理核心操作的异步接口，包括：
/// - 用户注册
/// - 用户登录
/// - 用户登出
///
/// # 设计说明
/// 1. 使用`async_trait`宏支持异步方法
/// 2. 所有方法返回`Result`，错误类型为`Box<dyn ApplicationError>`
/// 3. 接收命令对象作为参数，遵循CQRS模式
#[async_trait]
pub trait UserManagerService: 'static + Send + Sync {
    /// 注册新用户
    ///
    /// # Arguments
    /// * `command` - 用户注册命令
    ///
    /// # Errors
    /// 可能返回以下错误：
    /// - `UserManagerError::UserAlreadyExists`: 用户已存在
    /// - `UserManagerError::InvalidUsernameFormat`: 无效的用户名格式
    /// - `UserManagerError::InvalidPasswordFormat`: 无效的密码格式
    /// - `UserManagerError::InvalidNameFormat`: 无效的姓名格式
    async fn register(&self, command: UserRegisterCommand)
    -> Result<(), Box<dyn ApplicationError>>;
    /// 用户登录
    ///
    /// # Arguments
    /// * `command` - 用户登录命令
    ///
    /// # Returns
    /// 成功时返回新创建的会话ID
    ///
    /// # Errors
    /// 可能返回以下错误：
    /// - `UserManagerError::InvalidPhoneNumberOrPassword`: 无效的手机号或密码
    async fn login(
        &self,
        command: UserLoginCommand,
    ) -> Result<SessionId, Box<dyn ApplicationError>>;
    /// 用户登出
    ///
    /// # Arguments
    /// * `command` - 用户登出命令
    ///
    /// # Errors
    /// 可能返回以下错误：
    /// - `GeneralError::InvalidSessionId`: 无效的会话ID
    async fn logout(&self, command: UserLogoutCommand) -> Result<(), Box<dyn ApplicationError>>;

    async fn update_password(
        &self,
        command: UserUpdatePasswordCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}
