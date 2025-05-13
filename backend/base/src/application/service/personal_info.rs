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
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 个人信息数据传输对象(DTO)
///
/// 用于在应用层和表示层之间传输个人信息数据。
/// 实现了从领域模型[`PersonalInfo`]到DTO的转换。
///
/// # Fields
/// - `name`: 用户真实姓名
/// - `identity_card_id`: 身份证号
/// - `preferred_seat_location`: 优先座位位置
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct PersonalInfoDTO {
    pub name: String,
    #[serde(rename = "identityCardId")]
    pub identity_card_id: String,
    #[serde(rename = "preferredSeatLocation")]
    pub preferred_seat_location: String,
}

/// 设置个人信息数据传输对象(DTO)
///
/// 用于接收客户端发送的个人信息更新请求。
///
/// # Fields
/// - `name`: 用户真实姓名
/// - `identity_card_id`: 身份证号
/// - `preferred_seat_location`: 优先座位位置
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct SetPersonalInfoDTO {
    pub name: String,
    #[serde(rename = "identityCardId")]
    pub identity_card_id: String,
    #[serde(rename = "preferredSeatLocation")]
    pub preferred_seat_location: String,
}

impl From<PersonalInfo> for PersonalInfoDTO {
    fn from(value: PersonalInfo) -> Self {
        PersonalInfoDTO {
            name: value.name().to_string(),
            identity_card_id: value.identity_card_id().to_string(),
            preferred_seat_location: value.preferred_seat_location().to_string(),
        }
    }
}

/// 个人信息服务接口
///
/// 定义了获取和更新个人信息的核心操作，所有实现都应该是线程安全的。
///
/// # Methods
/// - `get_personal_info`: 根据查询条件获取个人信息
/// - `set_personal_info`: 根据命令更新个人信息
///
/// # Errors
/// 操作可能返回以下错误：
/// - `PersonalInfoError::InvalidSessionId`: 当会话无效时
/// - `PersonalInfoError::BadRequest`: 当请求参数无效时
/// - `PersonalInfoError::InternalServerError`: 当服务器内部错误时
#[async_trait]
pub trait PersonalInfoService: 'static + Send + Sync {
    async fn get_personal_info(
        &self,
        query: PersonalInfoQuery,
    ) -> Result<PersonalInfoDTO, Box<dyn ApplicationError>>;

    async fn set_personal_info(
        &self,
        command: SetPersonalInfoCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}

#[derive(Error, Debug)]
pub enum PersonalInfoError {
    #[error("Invalid identity card ID")]
    InvalidIdentityCardId,
    #[error("Invalid preferred seat location")]
    InvalidPreferredSeatLocation,
}

impl ApplicationError for PersonalInfoError {
    fn error_code(&self) -> u32 {
        match self {
            Self::InvalidIdentityCardId => 400,
            Self::InvalidPreferredSeatLocation => 400,
        }
    }

    fn error_message(&self) -> String {
        match self {
            Self::InvalidIdentityCardId => "Invalid identity card ID".to_string(),
            Self::InvalidPreferredSeatLocation => "Invalid preferred seat location".to_string(),
        }
    }
}