// We will never forget those who fell in the defense of Malevelon Creek.

pub mod queue;

use crate::MicroService;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use uuid::Uuid;

pub const RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME: &str = "super_event_queue";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Event {
    ForSuperEarth,
    CityUpdatedEvent,
    StationUpdatedEvent,
    UserUpdatedEvent,
    PersonalInfoUpdatedEvent,
    TrainUpdatedEvent,
    TrainScheduleUpdatedEvent,
    RouteUpdatedEvent,
    SeatTypeUpdatedEvent,
    HotelUpdatedEvent,
    HotelRoomTypeUpdatedEvent,
    DishUpdatedEvent,
    TakeawayDishUpdatedEvent,
}

impl Event {
    /// 返回事件的下划线分隔、小写的名称。
    /// 例如：`CityUpdatedEvent` -> `city_updated`
    pub fn event_name(&self) -> &'static str {
        match self {
            Event::ForSuperEarth => "for_super_earth",
            Event::CityUpdatedEvent => "city_updated",
            Event::StationUpdatedEvent => "station_updated",
            Event::UserUpdatedEvent => "user_updated",
            Event::PersonalInfoUpdatedEvent => "personal_info_updated",
            Event::TrainUpdatedEvent => "train_updated",
            Event::TrainScheduleUpdatedEvent => "train_schedule_updated",
            Event::RouteUpdatedEvent => "route_updated",
            Event::SeatTypeUpdatedEvent => "seat_type_updated",
            Event::HotelUpdatedEvent => "hotel_updated",
            Event::HotelRoomTypeUpdatedEvent => "hotel_room_type_updated",
            Event::DishUpdatedEvent => "dish_updated",
            Event::TakeawayDishUpdatedEvent => "takeaway_dish_updated",
        }
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.event_name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventPackage {
    pub id: Uuid,
    pub source: MicroService,
    pub time: DateTime<Utc>,
    pub name: String,
    pub event: Event,
}

impl EventPackage {
    pub fn new(source: MicroService, event: Event) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            time: Utc::now(),
            name: event.event_name().to_string(),
            event,
        }
    }

    pub fn topic_key(&self) -> String {
        format!("{}.{}", self.source, self.name)
    }
}

impl std::fmt::Display for EventPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.id, self.source, self.time, self.name, self.event
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_name_generation() {
        assert_eq!(Event::ForSuperEarth.event_name(), "for_super_earth");
        assert_eq!(Event::CityUpdatedEvent.event_name(), "city_updated");
        assert_eq!(Event::StationUpdatedEvent.event_name(), "station_updated");
        assert_eq!(Event::UserUpdatedEvent.event_name(), "user_updated");
        assert_eq!(
            Event::PersonalInfoUpdatedEvent.event_name(),
            "personal_info_updated"
        );
        assert_eq!(Event::TrainUpdatedEvent.event_name(), "train_updated");
        assert_eq!(
            Event::TrainScheduleUpdatedEvent.event_name(),
            "train_schedule_updated"
        );
        assert_eq!(Event::RouteUpdatedEvent.event_name(), "route_updated");
        assert_eq!(
            Event::SeatTypeUpdatedEvent.event_name(),
            "seat_type_updated"
        );
        assert_eq!(Event::HotelUpdatedEvent.event_name(), "hotel_updated");
        assert_eq!(
            Event::HotelRoomTypeUpdatedEvent.event_name(),
            "hotel_room_type_updated"
        );
        assert_eq!(Event::DishUpdatedEvent.event_name(), "dish_updated");
        assert_eq!(
            Event::TakeawayDishUpdatedEvent.event_name(),
            "takeaway_dish_updated"
        );
    }

    #[test]
    fn test_event_display_trait() {
        let event = Event::UserUpdatedEvent;
        assert_eq!(format!("{}", event), "user_updated");

        let event2 = Event::ForSuperEarth;
        assert_eq!(format!("{}", event2), "for_super_earth");
    }

    #[test]
    fn test_event_package_new() {
        let source = MicroService::Train;
        let event = Event::UserUpdatedEvent;
        let package = EventPackage::new(source, event);

        assert_eq!(package.source, source);
        assert_eq!(package.event, event);
        assert_eq!(package.name, "user_updated");

        // 检查时间是否在合理范围内（例如，在过去5秒内）
        let now = Utc::now();
        let duration = now.signed_duration_since(package.time);
        assert!(duration.num_seconds() < 5);
    }

    #[test]
    fn test_event_package_topic_key() {
        let source = MicroService::Train;
        let event = Event::TrainScheduleUpdatedEvent;
        let package = EventPackage::new(source, event);

        let expected_topic_key = format!("{}.{}", source, event.event_name());
        assert_eq!(package.topic_key(), expected_topic_key);
        assert_eq!(package.topic_key(), "train.train_schedule_updated");
    }

    #[test]
    fn test_event_package_display_trait() {
        let source = MicroService::Train;
        let event = Event::HotelUpdatedEvent;
        let mut package = EventPackage::new(source, event);

        // 为了进行确定性测试，我们手动设置ID和时间
        let fixed_id = Uuid::new_v4();
        let fixed_time = Utc::now();
        package.id = fixed_id;
        package.time = fixed_time;

        let display_str = format!("{}", package);
        let expected_str = format!(
            "{} {} {} {} {}",
            fixed_id,
            source,
            fixed_time,
            event.event_name(),
            event
        );

        assert_eq!(display_str, expected_str);
    }
}
