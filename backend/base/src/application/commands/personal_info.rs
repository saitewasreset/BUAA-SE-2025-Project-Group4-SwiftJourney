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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service::personal_info::SetPersonalInfoDTO;

    #[test]
    fn test_from_session_id_and_dto_positive() {
        let dto = SetPersonalInfoDTO {
            name: Some("张三".to_string()),
            identity_card_id: "1234567890".to_string(),
            preferred_seat_location: Some("窗口".to_string()),
            default: Some(true),
        };

        let cmd = SetPersonalInfoCommand::from_session_id_and_dto("session123".to_string(), dto.clone());

        assert_eq!(cmd.session_id, "session123");
        assert_eq!(cmd.name, dto.name);
        assert_eq!(cmd.identity_card_id, dto.identity_card_id);
        assert_eq!(cmd.preferred_seat_location, dto.preferred_seat_location);
        assert_eq!(cmd.default, dto.default);
    }

    #[test]
    fn test_from_session_id_and_dto_negative() {
        let dto = SetPersonalInfoDTO {
            name: None,
            identity_card_id: "9876543210".to_string(),
            preferred_seat_location: None,
            default: None,
        };

        let cmd = SetPersonalInfoCommand::from_session_id_and_dto("session999".to_string(), dto);

        assert_eq!(cmd.session_id, "session999");
        assert!(cmd.name.is_none());
        assert!(cmd.preferred_seat_location.is_none());
        assert!(cmd.default.is_none());
    }

    #[test]
    fn test_is_delete_operation_positive() {
        let cmd = SetPersonalInfoCommand {
            session_id: "session1".to_string(),
            name: None,
            identity_card_id: "123456".to_string(),
            preferred_seat_location: None,
            default: None,
        };

        assert!(cmd.is_delete_operation());
    }

    #[test]
    fn test_is_delete_operation_negative() {
        let cmd = SetPersonalInfoCommand {
            session_id: "session2".to_string(),
            name: Some("李四".to_string()),
            identity_card_id: "654321".to_string(),
            preferred_seat_location: None,
            default: None,
        };

        assert!(!cmd.is_delete_operation());
    }

    #[test]
    fn test_is_update_operation_positive() {
        let cmd = SetPersonalInfoCommand {
            session_id: "session3".to_string(),
            name: Some("王五".to_string()),
            identity_card_id: "111111".to_string(),
            preferred_seat_location: Some("过道".to_string()),
            default: Some(true),
        };

        assert!(cmd.is_update_operation());
    }

    #[test]
    fn test_is_update_operation_negative() {
        let cmd = SetPersonalInfoCommand {
            session_id: "session4".to_string(),
            name: None, // 缺少姓名
            identity_card_id: "222222".to_string(),
            preferred_seat_location: Some("中间".to_string()),
            default: Some(false),
        };

        assert!(!cmd.is_update_operation());
    }
}

