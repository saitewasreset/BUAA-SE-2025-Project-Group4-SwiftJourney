//! 用户管理命令模块
//!
//! 本模块定义了用户管理相关的命令(Command)结构体，这些结构体用于表示用户管理操作的具体请求数据。
//! 命令是从应用层DTO转换而来，包含执行操作所需的全部数据，将被传递给应用服务进行处理。
//!
//! ## 命令类型
//! - `UserRegisterCommand`: 用户注册命令
//! - `UserLoginCommand`: 用户登录命令
//! - `UserLogoutCommand`: 用户登出命令
//!
//! ## 设计原则
//! 1. 不可变性: 所有命令字段都是不可变的
//! 2. 明确性: 每个命令只包含执行该操作所需的最小数据集
//! 3. 值语义: 实现了`Clone`, `PartialEq`等trait便于测试和验证
use crate::application::service::user_manager::{UserLoginDTO, UserRegisterDTO};

/// 用户注册命令
///
/// 表示一个用户注册请求，包含注册所需的所有信息。
/// 该命令通常由`UserRegisterDTO`转换而来，然后传递给`UserManagerService`处理。
///
/// # Fields
/// - `phone`: 用户手机号码
/// - `username`: 用户名
/// - `password`: 用户密码(明文)
/// - `name`: 用户真实姓名
/// - `identity_card_id`: 用户身份证号码
///
/// # Examples
///
/// ```
/// use base::application::commands::user_manager::UserRegisterCommand;
/// use base::application::service::user_manager::UserRegisterDTO;
///
/// let dto = UserRegisterDTO {
///     phone: "13012345678".to_string(),
///     username: "For Super Earth!".to_string(),
///     password: "password123".to_string(),
///     name: "张三".to_string(),
///     identity_card_id: "11010519491231002X".to_string(),
/// };
///
/// let command = UserRegisterCommand::from(dto);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserRegisterCommand {
    pub phone: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub identity_card_id: String,
}

impl From<UserRegisterDTO> for UserRegisterCommand {
    fn from(dto: UserRegisterDTO) -> Self {
        UserRegisterCommand {
            phone: dto.phone,
            username: dto.username,
            password: dto.password,
            name: dto.name,
            identity_card_id: dto.identity_card_id,
        }
    }
}

/// 用户登录命令
///
/// 表示一个用户登录请求，包含登录凭证信息。
/// 该命令通常由`UserLoginDTO`转换而来，然后传递给`UserManagerService`处理。
///
/// # Fields
/// - `phone`: 用户手机号码(作为登录账号)
/// - `password`: 用户密码(明文)
///
/// # Examples
///
/// ```
/// use base::application::commands::user_manager::UserLoginCommand;
/// use base::application::service::user_manager::UserLoginDTO;
///
/// let dto = UserLoginDTO {
///     phone: "13012345678".to_string(),
///     password: "password123".to_string(),
/// };
///
/// let command = UserLoginCommand::from(dto);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLoginCommand {
    pub phone: String,
    pub password: String,
}

impl From<UserLoginDTO> for UserLoginCommand {
    fn from(value: UserLoginDTO) -> Self {
        UserLoginCommand {
            phone: value.phone,
            password: value.password,
        }
    }
}

/// 用户登出命令
///
/// 表示一个用户登出请求，包含要终止的会话信息。
///
/// # Fields
/// - `session_id`: 要注销的会话ID
///
/// # Examples
///
/// ```
/// use base::application::commands::user_manager::UserLogoutCommand;
///
/// let command = UserLogoutCommand {
///     session_id: "session-id-123".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLogoutCommand {
    pub session_id: String,
}
