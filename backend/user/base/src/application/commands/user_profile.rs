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
