//! 个人信息命令模块
//!
//! 该模块实现了个人信息相关的命令和查询数据结构，遵循CQRS模式。
//! 主要用于处理个人信息的更新和查询操作。
//!
//! # 主要结构
//! - [`PersonalInfoQuery`][]: 个人信息查询结构，包含查询所需参数
//! - [`SetPersonalInfoCommand`][]: 设置个人信息命令，包含更新所需参数
//!
//! # 转换
//! 提供了从DTO到命令的转换实现，便于从应用层接收数据后转换为命令对象。
//!
//! # 注意事项
//! - 所有命令和查询结构都实现了`Debug`、`Clone`等常用trait
//! - 命令转换过程中会保留所有DTO字段

use crate::application::service::personal_info::SetPersonalInfoDTO;

/// 个人信息查询结构
///
/// 用于查询个人信息，包含必要的会话标识。
///
/// # Fields
/// - `session_id`: 用户会话标识，用于验证查询权限
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersonalInfoQuery {
    pub session_id: String,
}

/// 设置个人信息命令
///
/// 包含更新或删除个人信息所需的所有字段。
/// - 若要更新/新增信息，需设置name、identityCardId、preferredSeatLocation和default字段
/// - 若要删除信息，只设置identityCardId字段
///
/// # Fields
/// - `session_id`: 用户会话标识，用于验证操作权限
/// - `name`: 用户真实姓名（仅更新/创建时需要）
/// - `identity_card_id`: 身份证号（必填）
/// - `preferred_seat_location`: 优先座位位置（可选）
/// - `default`: 是否为默认个人信息（仅更新/创建时需要）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetPersonalInfoCommand {
    pub session_id: String,
    pub name: Option<String>,
    pub identity_card_id: String,
    pub preferred_seat_location: Option<String>,
    pub default: Option<bool>,
}

impl SetPersonalInfoCommand {
    /// 从会话ID和DTO创建命令
    ///
    /// 将应用层传输的DTO转换为命令对象，保留所有字段值。
    ///
    /// # Arguments
    /// * `session_id` - 用户会话标识
    /// * `dto` - 包含个人信息更新数据的数据传输对象
    ///
    /// # Returns
    /// 返回构建好的[`SetPersonalInfoCommand`]实例
    pub fn from_session_id_and_dto(session_id: String, dto: SetPersonalInfoDTO) -> Self {
        SetPersonalInfoCommand {
            session_id,
            name: dto.name,
            identity_card_id: dto.identity_card_id,
            preferred_seat_location: dto.preferred_seat_location,
            default: dto.default,
        }
    }

    /// 判断是否为删除操作
    ///
    /// 如果只提供了身份证号，没有其他参数，则视为删除操作
    pub fn is_delete_operation(&self) -> bool {
        self.name.is_none() && self.preferred_seat_location.is_none() && self.default.is_none()
    }

    /// 判断是否为更新/创建操作
    ///
    /// 如果提供了姓名、身份证号和默认设置，则视为更新/创建操作
    pub fn is_update_operation(&self) -> bool {
        self.name.is_some() && self.default.is_some()
    }
}
