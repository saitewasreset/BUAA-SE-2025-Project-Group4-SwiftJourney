//! 用户会话领域模型
//!
//! 本模块定义了与会话(Session)相关的核心领域结构，包括：
//! - 会话唯一标识(`SessionId`)
//! - 会话实体(`Session`)
//! - 相关错误类型(`SessionIdError`)

use crate::domain::model::user::UserId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use chrono::{DateTime, Utc};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

/// 会话唯一标识符
///
/// 基于UUID v4实现，保证全局唯一性。
/// 实现了与字符串/UUID之间的转换能力。
///
/// # Examples
///
/// ```
/// use base::domain::model::session::SessionId;
///
/// let id = SessionId::random();
/// let str_id = id.to_string();
/// let parsed = SessionId::try_from(str_id.as_str()).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(Uuid);

impl Display for SessionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

/// 会话ID相关错误
#[derive(Error, Debug)]
pub enum SessionIdError {
    /// UUID格式无效时返回此错误
    #[error("Invalid UUID format")]
    InvalidUuidFormat(#[from] uuid::Error),
}

impl Identifier for SessionId {}

impl SessionId {
    /// 生成随机会话IDc
    pub fn random() -> Self {
        SessionId(Uuid::new_v4())
    }
}

impl From<Uuid> for SessionId {
    fn from(value: Uuid) -> Self {
        SessionId(value)
    }
}

impl From<SessionId> for Uuid {
    fn from(value: SessionId) -> Self {
        value.0
    }
}

impl TryFrom<&str> for SessionId {
    type Error = SessionIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let uuid = Uuid::parse_str(value)?;
        Ok(SessionId(uuid))
    }
}

/// 用户会话实体
///
/// 表示用户的一次认证会话，包含：
/// - 唯一标识(ID)
/// - 关联用户ID
/// - 创建时间
/// - 过期时间
///
/// # 生命周期
///
/// 会话通过`is_expired()`方法检查是否过期
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Session {
    id: SessionId,
    user_id: UserId,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl Identifiable for Session {
    type ID = SessionId;
    fn get_id(&self) -> Option<Self::ID> {
        Some(self.id)
    }
}

impl Entity for Session {}
impl Aggregate for Session {}

impl Session {
    /// 创建新会话
    ///
    /// # Arguments
    /// - `user_id`: 关联用户ID
    /// - `created_at`: 会话创建时间
    /// - `expires_at`: 会话过期时间
    pub fn new(user_id: UserId, created_at: DateTime<Utc>, expires_at: DateTime<Utc>) -> Session {
        Session {
            id: SessionId::random(),
            user_id,
            created_at,
            expires_at,
        }
    }

    /// 获取会话ID
    pub fn session_id(&self) -> SessionId {
        self.id
    }

    /// 获取关联用户ID
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// 检查会话是否已过期
    ///
    /// 根据系统当前时间与`expires_at`比较
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
