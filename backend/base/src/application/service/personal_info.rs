//! 个人信息服务模块
//!
//! 该模块提供个人信息相关的服务接口和数据结构，包括：
//! - 个人信息数据的查询和更新
//! - 个人信息DTO的定义和转换
//! - 个人信息操作相关的错误类型
//!
//! 主要功能通过[`PersonalInfoService`] trait定义，包含获取和设置个人信息的方法。

use crate::application::ApplicationError;
use crate::application::commands::personal_info::{PersonalInfoQuery, SetPersonalInfoCommand};
use crate::domain::model::personal_info::PersonalInfo;
use crate::domain::Identifiable;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 个人信息数据传输对象(DTO)
///
/// 用于在应用层和表示层之间传输个人信息数据。
/// 实现了从领域模型[`PersonalInfo`]到DTO的转换。
///
/// # Fields
/// - `personal_id`: 个人信息ID
/// - `name`: 用户真实姓名
/// - `identity_card_id`: 身份证号
/// - `preferred_seat_location`: 优先座位位置
/// - `default`: 是否为默认个人信息
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PersonalInfoDTO {
    #[serde(rename = "personalId")]
    pub personal_id: String,
    pub name: String,
    #[serde(rename = "identityCardId")]
    pub identity_card_id: String,
    #[serde(rename = "preferredSeatLocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_seat_location: Option<String>,
    pub default: bool,
}

/// 设置个人信息数据传输对象(DTO)
///
/// 用于接收客户端发送的个人信息更新请求。
///
/// # Fields
/// - `name`: 用户真实姓名（只在创建/更新时需要设置）
/// - `identity_card_id`: 身份证号（必填）
/// - `preferred_seat_location`: 优先座位位置（可选）
/// - `default`: 是否为默认个人信息（只在创建/更新时需要设置）
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SetPersonalInfoDTO {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "identityCardId")]
    pub identity_card_id: String,
    #[serde(rename = "preferredSeatLocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_seat_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}

impl From<PersonalInfo> for PersonalInfoDTO {
    fn from(value: PersonalInfo) -> Self {
        PersonalInfoDTO {
            personal_id: value
                .get_id()
                .map_or_else(|| "".to_string(), |id| id.to_string()),
            name: value.name().to_string(),
            identity_card_id: value.identity_card_id().to_string(),
            preferred_seat_location: Some(value.preferred_seat_location().to_string()),
            default: value.is_default(),
        }
    }
}

/// 个人信息服务接口
///
/// 定义了获取和更新个人信息的核心操作，所有实现都应该是线程安全的。
///
/// # Methods
/// - `get_personal_info`: 获取用户所有个人信息列表
/// - `set_personal_info`: 更新或删除指定的个人信息
///
/// # Errors
/// 操作可能返回以下错误：
/// - `PersonalInfoError::InvalidIdentityCardIdFormat`: 身份证号格式错误
/// - `PersonalInfoError::InvalidIdentityCardId`: 身份证号对应的个人信息不存在
/// - `PersonalInfoError::InvalidPreferredSeatLocation`: 无效的座位偏好
/// - `GeneralError::InvalidSessionId`: 会话无效
/// - `GeneralError::InternalServerError`: 内部服务错误
#[async_trait]
pub trait PersonalInfoService: 'static + Send + Sync {
    async fn get_personal_info(
        &self,
        query: PersonalInfoQuery,
    ) -> Result<Vec<PersonalInfoDTO>, Box<dyn ApplicationError>>;

    async fn set_personal_info(
        &self,
        command: SetPersonalInfoCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}

#[derive(Error, Debug)]
pub enum PersonalInfoError {
    #[error("Identity card id format")]
    InvalidIdentityCardIdFormat,

    #[error("Invalid identity card id")]
    InvalidIdentityCardId,

    #[error("Invalid preferred seat location")]
    InvalidPreferredSeatLocation,
}

impl ApplicationError for PersonalInfoError {
    fn error_code(&self) -> u32 {
        match self {
            Self::InvalidIdentityCardIdFormat => 13001,
            Self::InvalidIdentityCardId => 13002,
            Self::InvalidPreferredSeatLocation => 13003,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
