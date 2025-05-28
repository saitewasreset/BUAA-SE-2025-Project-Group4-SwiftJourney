use crate::domain::Identifier;
use crate::domain::model::order::Order;
use crate::domain::model::user::UserId;
use chrono::Local;
use dyn_clone::{DynClone, clone_trait_object};
use id_macro::define_id_type;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::any::Any;
use std::fmt::{Debug, Display};

define_id_type!(Notify);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotifyType {
    Order,
    Trip,
}

impl Display for NotifyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotifyType::Order => write!(f, "order"),
            NotifyType::Trip => write!(f, "trip"),
        }
    }
}

impl TryFrom<&str> for NotifyType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "order" => Ok(NotifyType::Order),
            "trip" => Ok(NotifyType::Trip),
            _ => Err(format!("Invalid NotifyType: {}", value)),
        }
    }
}

pub trait Notify: DynClone + Debug + 'static + Send + Sync + Any {
    fn notify_id(&self) -> Option<NotifyId>;

    fn set_notify_id(&mut self, notify_id: NotifyId);

    fn user_id(&self) -> UserId;
    fn title(&self) -> &str;
    fn message_time(&self) -> DateTimeWithTimeZone;
    fn notify_type(&self) -> NotifyType;
}

clone_trait_object!(Notify);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BaseNotify {
    notify_id: Option<NotifyId>,
    user_id: UserId,
    title: String,
    message_time: DateTimeWithTimeZone,
    notify_type: NotifyType,
}

impl BaseNotify {
    pub fn new(
        notify_id: Option<NotifyId>,
        user_id: UserId,
        title: String,
        message_time: DateTimeWithTimeZone,
        notify_type: NotifyType,
    ) -> Self {
        BaseNotify {
            notify_id,
            user_id,
            title,
            message_time,
            notify_type,
        }
    }

    pub fn new_now(
        notify_id: Option<NotifyId>,
        user_id: UserId,
        title: String,
        notify_type: NotifyType,
    ) -> Self {
        let local_now = Local::now();
        let offset = *local_now.offset(); // 获取系统当前时区偏移
        let now = local_now.with_timezone(&offset);

        Self::new(notify_id, user_id, title, now, notify_type)
    }
}

#[derive(Clone, Debug)]
pub struct OrderNotify {
    base: BaseNotify,
    order: Box<dyn Order>,
}

impl OrderNotify {
    pub fn new(
        notify_id: Option<NotifyId>,
        user_id: UserId,
        title: String,
        message_time: DateTimeWithTimeZone,
        order: Box<dyn Order>,
    ) -> Self {
        let base = BaseNotify::new(notify_id, user_id, title, message_time, NotifyType::Order);

        OrderNotify { base, order }
    }

    pub fn new_now(user_id: UserId, title: String, order: Box<dyn Order>) -> Self {
        let base = BaseNotify::new_now(None, user_id, title, NotifyType::Order);

        OrderNotify { base, order }
    }

    pub fn order(&self) -> &dyn Order {
        self.order.as_ref()
    }
}

impl Notify for OrderNotify {
    fn notify_id(&self) -> Option<NotifyId> {
        self.base.notify_id
    }

    fn set_notify_id(&mut self, notify_id: NotifyId) {
        self.base.notify_id = Some(notify_id);
    }

    fn user_id(&self) -> UserId {
        self.base.user_id
    }

    fn title(&self) -> &str {
        &self.base.title
    }

    fn message_time(&self) -> DateTimeWithTimeZone {
        self.base.message_time
    }

    fn notify_type(&self) -> NotifyType {
        self.base.notify_type
    }
}

#[derive(Clone, Debug)]
pub struct TripNotify {
    base: BaseNotify,
    train_number: String,
    departure_time: DateTimeWithTimeZone,
    departure_station: String,
    arrival_station: String,
}

impl TripNotify {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        notify_id: Option<NotifyId>,
        user_id: UserId,
        title: String,
        message_time: DateTimeWithTimeZone,
        train_number: String,
        departure_time: DateTimeWithTimeZone,
        departure_station: String,
        arrival_station: String,
    ) -> Self {
        let base = BaseNotify::new(notify_id, user_id, title, message_time, NotifyType::Trip);

        TripNotify {
            base,
            train_number,
            departure_time,
            departure_station,
            arrival_station,
        }
    }

    pub fn new_now(
        user_id: UserId,
        title: String,
        train_number: String,
        departure_time: DateTimeWithTimeZone,
        departure_station: String,
        arrival_station: String,
    ) -> Self {
        let base = BaseNotify::new_now(None, user_id, title, NotifyType::Trip);

        TripNotify {
            base,
            train_number,
            departure_time,
            departure_station,
            arrival_station,
        }
    }

    pub fn train_number(&self) -> &str {
        &self.train_number
    }

    pub fn departure_time(&self) -> DateTimeWithTimeZone {
        self.departure_time
    }

    pub fn departure_station(&self) -> &str {
        &self.departure_station
    }

    pub fn arrival_station(&self) -> &str {
        &self.arrival_station
    }
}

impl Notify for TripNotify {
    fn notify_id(&self) -> Option<NotifyId> {
        self.base.notify_id
    }

    fn set_notify_id(&mut self, notify_id: NotifyId) {
        self.base.notify_id = Some(notify_id);
    }

    fn user_id(&self) -> UserId {
        self.base.user_id
    }

    fn title(&self) -> &str {
        &self.base.title
    }

    fn message_time(&self) -> DateTimeWithTimeZone {
        self.base.message_time
    }

    fn notify_type(&self) -> NotifyType {
        self.base.notify_type
    }
}
