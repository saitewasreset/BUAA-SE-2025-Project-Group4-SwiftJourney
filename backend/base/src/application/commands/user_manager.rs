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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserUpdatePasswordCommand {
    pub session_id: String,
    pub origin_password: String,
    pub new_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service::user_manager::{UserRegisterDTO, UserLoginDTO};

    // ------------------------------
    // UserRegisterCommand 测试
    // ------------------------------
    #[test]
    fn test_user_register_from_dto_positive() {
        let dto = UserRegisterDTO {
            phone: "13012345678".to_string(),
            username: "testuser".to_string(),
            password: "password123".to_string(),
            name: "张三".to_string(),
            identity_card_id: "11010519491231002X".to_string(),
        };

        let cmd = UserRegisterCommand::from(dto.clone());
        assert_eq!(cmd.phone, dto.phone);
        assert_eq!(cmd.username, dto.username);
        assert_eq!(cmd.password, dto.password);
        assert_eq!(cmd.name, dto.name);
        assert_eq!(cmd.identity_card_id, dto.identity_card_id);
    }

    #[test]
    fn test_user_register_from_dto_negative_empty_fields() {
        let dto = UserRegisterDTO {
            phone: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            name: "".to_string(),
            identity_card_id: "".to_string(),
        };

        let cmd = UserRegisterCommand::from(dto);
        assert!(cmd.phone.is_empty());
        assert!(cmd.username.is_empty());
        assert!(cmd.password.is_empty());
        assert!(cmd.name.is_empty());
        assert!(cmd.identity_card_id.is_empty());
    }

    // ------------------------------
    // UserLoginCommand 测试
    // ------------------------------
    #[test]
    fn test_user_login_from_dto_positive() {
        let dto = UserLoginDTO {
            phone: "13012345678".to_string(),
            password: "password123".to_string(),
        };

        let cmd = UserLoginCommand::from(dto.clone());
        assert_eq!(cmd.phone, dto.phone);
        assert_eq!(cmd.password, dto.password);
    }

    #[test]
    fn test_user_login_from_dto_negative_empty_password() {
        let dto = UserLoginDTO {
            phone: "13000000000".to_string(),
            password: "".to_string(),
        };

        let cmd = UserLoginCommand::from(dto);
        assert_eq!(cmd.phone, "13000000000");
        assert!(cmd.password.is_empty());
    }

    // ------------------------------
    // UserLogoutCommand 测试
    // ------------------------------
    #[test]
    fn test_user_logout_command_positive() {
        let cmd = UserLogoutCommand {
            session_id: "session123".to_string(),
        };
        assert_eq!(cmd.session_id, "session123");
    }

    #[test]
    fn test_user_logout_command_negative_empty_session() {
        let cmd = UserLogoutCommand {
            session_id: "".to_string(),
        };
        assert!(cmd.session_id.is_empty());
    }

    // ------------------------------
    // UserUpdatePasswordCommand 测试
    // ------------------------------
    #[test]
    fn test_user_update_password_command_positive() {
        let cmd = UserUpdatePasswordCommand {
            session_id: "session999".to_string(),
            origin_password: "oldpass".to_string(),
            new_password: "newpass".to_string(),
        };

        assert_eq!(cmd.session_id, "session999");
        assert_eq!(cmd.origin_password, "oldpass");
        assert_eq!(cmd.new_password, "newpass");
    }

    #[test]
    fn test_user_update_password_command_negative_empty_passwords() {
        let cmd = UserUpdatePasswordCommand {
            session_id: "s".to_string(),
            origin_password: "".to_string(),
            new_password: "".to_string(),
        };

        assert!(cmd.origin_password.is_empty());
        assert!(cmd.new_password.is_empty());
    }
}

