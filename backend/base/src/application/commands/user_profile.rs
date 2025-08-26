//! 用户资料命令模块
//!
//! 该模块实现了用户资料相关的命令和查询数据结构，遵循CQRS模式。
//! 主要用于处理用户资料的更新和查询操作。
//!
//! # 主要结构
//! - [`UserProfileQuery`][]: 用户资料查询结构，包含查询所需参数
//! - [`SetUserProfileCommand`][]: 设置用户资料命令，包含更新所需参数
//!
//! # 转换
//! 提供了从DTO到命令的转换实现，便于从应用层接收数据后转换为命令对象。
//!
//! # 注意事项
//! - 所有命令和查询结构都实现了`Debug`、`Clone`等常用trait
//! - 命令转换过程中会保留所有DTO字段
use crate::application::service::user_profile::SetUserProfileDTO;

/// 用户资料查询结构
///
/// 用于查询用户资料信息，包含必要的会话标识。
///
/// # Fields
/// - `session_id`: 用户会话标识，用于验证查询权限
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserProfileQuery {
    pub session_id: String,
}

/// 设置用户资料命令
///
/// 包含更新用户资料所需的所有字段，通过[`crate::application::service::user_profile::UserProfileService`]执行。
///
/// # Fields
/// - `session_id`: 用户会话标识，用于验证操作权限
/// - `username`: 用户名
/// - `gender`: 性别(可选)
/// - `age`: 年龄(可选)
/// - `email`: 电子邮箱
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetUserProfileCommand {
    pub session_id: String,
    pub username: String,
    pub gender: Option<String>,
    pub age: Option<u16>,
    pub email: String,
}

impl SetUserProfileCommand {
    /// 从会话ID和DTO创建命令
    ///
    /// 将应用层传输的DTO转换为命令对象，保留所有字段值。
    ///
    /// # Arguments
    /// * `session_id` - 用户会话标识
    /// * `dto` - 包含用户资料更新数据的数据传输对象
    ///
    /// # Returns
    /// 返回构建好的[`SetUserProfileCommand`]实例
    pub fn from_session_id_and_dto(session_id: String, dto: SetUserProfileDTO) -> Self {
        SetUserProfileCommand {
            session_id,
            username: dto.username,
            gender: dto.gender,
            age: dto.age,
            email: dto.email,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service::user_profile::SetUserProfileDTO;

    // ------------------------------
    // SetUserProfileCommand::from_session_id_and_dto 测试
    // ------------------------------

    #[test]
    fn test_from_session_id_and_dto_positive() {
        let dto = SetUserProfileDTO {
            username: "张三".to_string(),
            gender: Some("男".to_string()),
            age: Some(28),
            email: "zhangsan@example.com".to_string(),
        };

        let cmd = SetUserProfileCommand::from_session_id_and_dto("session123".to_string(), dto.clone());

        assert_eq!(cmd.session_id, "session123");
        assert_eq!(cmd.username, dto.username);
        assert_eq!(cmd.gender, dto.gender);
        assert_eq!(cmd.age, dto.age);
        assert_eq!(cmd.email, dto.email);
    }

    #[test]
    fn test_from_session_id_and_dto_negative_empty_fields() {
        let dto = SetUserProfileDTO {
            username: "".to_string(),
            gender: None,
            age: None,
            email: "".to_string(),
        };

        let cmd = SetUserProfileCommand::from_session_id_and_dto("".to_string(), dto);

        assert!(cmd.session_id.is_empty());
        assert!(cmd.username.is_empty());
        assert!(cmd.gender.is_none());
        assert!(cmd.age.is_none());
        assert!(cmd.email.is_empty());
    }

    // ------------------------------
    // UserProfileQuery 测试
    // ------------------------------

    #[test]
    fn test_user_profile_query_positive() {
        let query = UserProfileQuery {
            session_id: "session999".to_string(),
        };

        assert_eq!(query.session_id, "session999");
    }

    #[test]
    fn test_user_profile_query_negative_empty_session() {
        let query = UserProfileQuery {
            session_id: "".to_string(),
        };

        assert!(query.session_id.is_empty());
    }
}
