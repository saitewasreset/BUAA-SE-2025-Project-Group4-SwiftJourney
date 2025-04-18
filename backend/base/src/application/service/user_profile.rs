//! 用户资料服务模块
//!
//! 该模块提供用户资料相关的服务接口和数据结构，包括：
//! - 用户资料数据的查询和更新
//! - 用户资料DTO的定义和转换
//! - 用户资料操作相关的错误类型
//!
//! 主要功能通过[`UserProfileService`] trait定义，包含获取和设置用户资料的方法。

use crate::application::ApplicationError;
use crate::application::commands::user_profile::{SetUserProfileCommand, UserProfileQuery};
use crate::domain::model::user::User;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 用户资料数据传输对象(DTO)
///
/// 用于在应用层和表示层之间传输用户资料数据，包含用户的基本信息和身份信息。
/// 实现了从领域模型[`User`]到DTO的转换。
///
/// # Fields
/// - `username`: 用户名
/// - `gender`: 性别(可选)
/// - `age`: 年龄(可选)
/// - `phone`: 手机号码
/// - `email`: 电子邮箱(可选)
/// - `have_payment_password_set`: 是否设置了支付密码
/// - `name`: 真实姓名
/// - `identity_card_id`: 身份证号
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct UserProfileDTO {
    pub username: String,
    pub gender: Option<String>,
    pub age: Option<u16>,
    pub phone: String,
    pub email: Option<String>,
    #[serde(rename = "havePaymentPasswordSet")]
    pub have_payment_password_set: bool,
    pub name: String,
    #[serde(rename = "identityCardId")]
    pub identity_card_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct SetUserProfileDTO {
    pub username: String,
    pub gender: Option<String>,
    pub age: Option<u16>,
    pub email: String,
}

impl From<User> for UserProfileDTO {
    fn from(value: User) -> Self {
        UserProfileDTO {
            username: value.username().to_owned(),
            gender: value.user_info().gender.map(|gender| gender.to_string()),
            age: value.user_info().age.map(|age| age.into()),
            phone: value.user_info().phone.to_string(),
            email: value
                .user_info()
                .email
                .as_ref()
                .map(|email| email.to_string()),
            have_payment_password_set: value.hashed_payment_password().is_some(),
            name: value.user_info().name.to_string(),
            identity_card_id: value.user_info().identity_card_id.to_string(),
        }
    }
}

/// 用户资料服务接口
///
/// 定义了获取和更新用户资料的核心操作，所有实现都应该是线程安全的。
///
/// # Methods
/// - `get_profile`: 根据查询条件获取用户资料
/// - `set_profile`: 根据命令更新用户资料
///
/// # Errors
/// 操作可能返回以下错误：
/// - `UserProfileError::InvalidSessionId`: 当会话无效时
/// - `UserProfileError::BadRequest`: 当请求参数无效时
/// - `UserProfileError::InternalServerError`: 当服务器内部错误时

#[async_trait]
pub trait UserProfileService {
    async fn get_profile(
        &self,
        query: UserProfileQuery,
    ) -> Result<UserProfileDTO, Box<dyn ApplicationError>>;

    async fn set_profile(
        &self,
        command: SetUserProfileCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}
