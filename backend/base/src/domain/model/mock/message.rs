#![cfg(test)]

use crate::domain::model::message::{Notify, NotifyId, NotifyType};
use crate::domain::model::user::UserId;
use chrono::Utc;
use std::fmt::Debug;
use sea_orm::prelude::DateTimeWithTimeZone;

// 创建一个 MockNotify 类型，实现 Notify trait
#[derive(Clone, Debug)]
pub struct MockNotify {
    notify_id: Option<NotifyId>,
    user_id: UserId,
    title: String,
    message_time: chrono::DateTime<Utc>,
    notify_type: NotifyType,
}

impl MockNotify {
    pub fn new(user_id: UserId, title: &str, notify_type: NotifyType) -> Self {
        Self {
            notify_id: None,
            user_id,
            title: title.to_string(),
            message_time: Utc::now(),
            notify_type,
        }
    }

    pub fn with_id(mut self, id: NotifyId) -> Self {
        self.notify_id = Some(id);
        self
    }
}

impl Notify for MockNotify {
    fn notify_id(&self) -> Option<NotifyId> {
        self.notify_id
    }

    fn set_notify_id(&mut self, notify_id: NotifyId) {
        self.notify_id = Some(notify_id);
    }

    fn user_id(&self) -> UserId {
        self.user_id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn message_time(&self) -> DateTimeWithTimeZone {
        DateTimeWithTimeZone::from(self.message_time)
    }

    fn notify_type(&self) -> NotifyType {
        self.notify_type
    }
}
