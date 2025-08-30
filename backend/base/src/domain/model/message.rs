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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::order::{
        BaseOrder, Order, OrderStatus, OrderTimeInfo, OrderType, PaymentInfo,
    };
    use crate::domain::model::personal_info::PersonalInfoId;
    use claims::{assert_err, assert_ok};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    // 伪实现一个最小的订单用于测试 OrderNotify
    #[derive(Debug, Clone)]
    struct DummyOrder(BaseOrder);

    impl Order for DummyOrder {
        fn order_id(&self) -> Option<crate::domain::model::order::OrderId> {
            self.0.order_id
        }
        fn uuid(&self) -> Uuid {
            self.0.uuid
        }
        fn already_refund(&self) -> bool {
            self.0.payment_info.refund_transaction_id().is_some()
        }
        fn order_status(&self) -> OrderStatus {
            self.0.order_status
        }
        fn order_type(&self) -> OrderType {
            OrderType::Train
        }
        fn order_time_info(&self) -> OrderTimeInfo {
            self.0.order_time_info
        }
        fn unit_price(&self) -> Decimal {
            self.0.unit_price
        }
        fn amount(&self) -> Decimal {
            self.0.amount
        }
        fn payment_info(&self) -> PaymentInfo {
            self.0.payment_info
        }
        fn payment_info_mut(&mut self) -> &mut PaymentInfo {
            &mut self.0.payment_info
        }
        fn personal_info_id(&self) -> PersonalInfoId {
            self.0.personal_info_id
        }
        fn set_status(&mut self, status: OrderStatus) {
            self.0.order_status = status;
        }
    }

    fn base_order_fixture() -> BaseOrder {
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        BaseOrder::new(
            None,
            Uuid::nil(),
            OrderStatus::Unpaid,
            OrderTimeInfo::new(now, now, now),
            Decimal::new(1000, 2),
            Decimal::new(1, 0),
            PaymentInfo::new(None, None),
            PersonalInfoId::from(1_u64),
        )
    }

    #[test]
    fn notify_type_try_from_and_display() {
        assert_ok!(NotifyType::try_from("order"));
        assert_ok!(NotifyType::try_from("trip"));
        assert_err!(NotifyType::try_from("other"));

        assert_eq!(NotifyType::Order.to_string(), "order");
        assert_eq!(NotifyType::Trip.to_string(), "trip");
    }

    #[test]
    fn base_notify_new_and_new_now() {
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        let user_id = UserId::from(42_u64);
        let base = BaseNotify::new(
            Some(NotifyId::from(7_u64)),
            user_id,
            "title".into(),
            now,
            NotifyType::Trip,
        );
        assert_eq!(base.user_id, user_id);
        assert_eq!(base.title, "title");
        assert_eq!(base.message_time, now);
        assert_eq!(base.notify_type, NotifyType::Trip);

        let base_now = BaseNotify::new_now(None, user_id, "t".into(), NotifyType::Order);
        assert_eq!(base_now.user_id, user_id);
        assert_eq!(base_now.title, "t");
        assert_eq!(base_now.notify_type, NotifyType::Order);
    }

    #[test]
    fn order_notify_behaviors() {
        let user_id = UserId::from(1_u64);
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        let order: Box<dyn Order> = Box::new(DummyOrder(base_order_fixture()));

        let mut notify =
            OrderNotify::new(Some(NotifyId::from(2_u64)), user_id, "o".into(), now, order);
        assert_eq!(notify.notify_id(), Some(NotifyId::from(2_u64)));
        assert_eq!(notify.user_id(), user_id);
        assert_eq!(notify.title(), "o");
        assert_eq!(notify.message_time(), now);
        assert_eq!(notify.notify_type(), NotifyType::Order);
        // set id
        notify.set_notify_id(NotifyId::from(3_u64));
        assert_eq!(notify.notify_id(), Some(NotifyId::from(3_u64)));

        let notify_now = OrderNotify::new_now(
            user_id,
            "n".into(),
            Box::new(DummyOrder(base_order_fixture())),
        );
        assert_eq!(notify_now.notify_type(), NotifyType::Order);
    }

    #[test]
    fn trip_notify_behaviors() {
        let user_id = UserId::from(9_u64);
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        let tn = "G100".to_string();
        let dep = now;
        let dep_sta = "Beijing".to_string();
        let arr_sta = "Shanghai".to_string();

        let mut notify = TripNotify::new(
            Some(NotifyId::from(5_u64)),
            user_id,
            "t".into(),
            now,
            tn.clone(),
            dep,
            dep_sta.clone(),
            arr_sta.clone(),
        );
        assert_eq!(notify.notify_id(), Some(NotifyId::from(5_u64)));
        assert_eq!(notify.user_id(), user_id);
        assert_eq!(notify.title(), "t");
        assert_eq!(notify.message_time(), now);
        assert_eq!(notify.notify_type(), NotifyType::Trip);
        assert_eq!(notify.train_number(), tn);
        assert_eq!(notify.departure_time(), dep);
        assert_eq!(notify.departure_station(), dep_sta);
        assert_eq!(notify.arrival_station(), arr_sta);

        notify.set_notify_id(NotifyId::from(6_u64));
        assert_eq!(notify.notify_id(), Some(NotifyId::from(6_u64)));

        let notify_now = TripNotify::new_now(
            user_id,
            "t2".into(),
            "D1".into(),
            dep,
            "A".into(),
            "B".into(),
        );
        assert_eq!(notify_now.notify_type(), NotifyType::Trip);
    }
}
