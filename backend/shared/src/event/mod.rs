// We will never forget those who fell in the defense of Malevelon Creek.

pub mod queue;

use crate::MicroService;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use uuid::Uuid;

/// RabbitMQ 中用于事件队列的 Topic Exchange 名称。
pub const RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME: &str = "super_event_queue";

/// 事件类型的字符串别名，用于在 `EventRegistry` 中唯一标识一个事件。
type EventType = String;

/// `Event` Trait 是所有领域事件必须实现的契约。
///
/// 它为事件提供了必要的元数据，以便在系统中进行序列化、网络传输和动态分发。
///
/// # Trait Bounds
/// - `Any`: 用于支持运行时类型识别和向 `dyn Any` 的转换，是动态分发的核心。
/// - `Send + Sync`: 确保事件可以在线程间安全地传递。
/// - `Clone`: 允许事件被复制，在某些场景下很有用。
/// - `Debug`: 便于日志记录和调试。
/// - `Serialize + DeserializeOwned`: 使事件能够被序列化为通用格式（如 JSON）进行网络传输，并能被反序列化回来。
///
/// # 方法
/// - `fn event_type() -> EventType`: 一个静态方法，返回该事件类型的唯一字符串标识符。
///   这个标识符将被用于序列化和在 `EventRegistry` 中查找对应的反序列化器。
///
/// # Example
/// ```ignore
/// use shared::event::{Event};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct UserCreatedEvent {
///     pub user_id: String,
///     pub username: String,
/// }
///
/// impl Event for UserCreatedEvent {
///     fn event_type() -> String {
///         "UserCreatedEvent".to_string()
///     }
/// }
/// ```
pub trait Event: Any + Send + Sync + Clone + Debug + Serialize + DeserializeOwned {
    /// 返回该事件类型的唯一字符串标识符。
    fn event_type() -> EventType;
}

/// `EventPackage` 是在网络中传输的事件“信封”。
///
/// 它包装了具体的领域事件，并附加上了必要的元数据，如事件ID、来源服务、时间戳等。
/// 这样做的好处是，接收方可以先解析 `EventPackage` 来了解事件的上下文，
/// 然后再根据 `name` 字段去反序列化内部具体的事件 `event`。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventPackage {
    /// 事件的唯一标识符。
    pub id: Uuid,
    /// 事件的来源微服务。
    pub source: MicroService,
    /// 事件发生的时间戳 (UTC)。
    pub time: DateTime<Utc>,
    /// 事件的类型名称，与 `Event::event_type()` 的返回值对应。
    pub name: EventType,
    /// 序列化后的具体事件内容。
    pub event: serde_json::Value,
}

impl EventPackage {
    /// 创建一个新的 `EventPackage`。
    ///
    /// 此方法会自动生成 UUID 和当前时间戳，并将传入的事件序列化为 `serde_json::Value`。
    ///
    /// # Arguments
    /// * `source`: 事件来源的微服务。
    /// * `event`: 实现了 `Event` trait 的具体事件实例。
    pub fn new<E: Event>(source: MicroService, event: E) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            time: Utc::now(),
            name: E::event_type(),
            event: serde_json::to_value(event).unwrap(),
        }
    }

    /// 生成用于 RabbitMQ Topic Exchange 的路由键 (routing key)。
    ///
    /// 格式为 `source_microservice.event_name`，例如 `UserService.UserCreatedEvent`。
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

/// 反序列化器函数的类型别名。
///
/// 它是一个函数指针，接受一个 `serde_json::Value`，并尝试将其反序列化为一个
/// `Box<dyn Any + Send + Sync>`，即一个类型被擦除但线程安全的事件对象。
type Deserializer = Box<
    dyn Fn(serde_json::Value) -> Result<Box<dyn Any + Send + Sync>, serde_json::Error>
        + Send
        + Sync,
>;

/// `EventRegistry` 是事件动态反序列化的核心。
///
/// 它维护一个从事件类型名称 (`EventType`) 到对应反序列化函数的映射。
/// 当从网络接收到一个 `EventPackage` 时，`EventRegistry` 可以根据包中的 `name` 字段，
/// 查找到正确的函数，将 `serde_json::Value` 格式的负载反序列化回原始的具体事件类型，
/// 并包装在 `Box<dyn Any>` 中。
pub struct EventRegistry {
    deserializers: HashMap<EventType, Deserializer>,
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRegistry {
    /// 创建一个新的、空的 `EventRegistry`。
    pub fn new() -> Self {
        Self {
            deserializers: HashMap::new(),
        }
    }

    /// 向注册中心注册一个事件类型。
    ///
    /// 此方法会为指定的事件类型 `E` 生成一个闭包，该闭包知道如何将 `serde_json::Value`
    /// 反序列化为 `E`，然后将此闭包存入内部的 `HashMap` 中。
    /// 每个微服务在启动时，都应该调用此方法来注册它关心或可能接收到的所有事件类型。
    ///
    /// # Type Parameters
    /// * `E`: 必须实现 `Event` trait 的具体事件类型。
    ///
    /// # Example
    /// ```ignore
    /// use shared::event::{Event, EventRegistry};
    /// // ... (假设 UserCreatedEvent 已定义并实现了 Event)
    ///
    /// let mut registry = EventRegistry::new();
    /// // registry.register::<UserCreatedEvent>();
    /// ```
    pub fn register<E: Event>(&mut self) {
        let event_type = E::event_type();

        let deserializer = Box::new(|value| {
            let event: E = serde_json::from_value(value)?;
            Ok(Box::new(event) as Box<dyn Any + Send + Sync>)
        });

        self.deserializers.insert(event_type, deserializer);
    }

    /// 根据 `EventPackage` 反序列化其内部的事件。
    ///
    /// # Returns
    /// - `Some(Ok(Box<dyn Any + ...>))`: 如果事件类型已知且反序列化成功。
    /// - `Some(Err(...))`: 如果事件类型已知但反序列化失败（例如 JSON 格式错误）。
    /// - `None`: 如果事件类型未在注册中心注册。
    pub fn deserialize(
        &self,
        event_package: &EventPackage,
    ) -> Option<Result<Box<dyn Any + Send + Sync>, serde_json::Error>> {
        self.deserializers
            .get(&event_package.name)
            .map(|deserializer| deserializer(event_package.event.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*; // 导入父模块的所有内容，包括 EventPackage, EventRegistry, Event 等
    use crate::MicroService; // 导入 MicroService 枚举
    use serde::{Deserialize, Serialize};

    // --- Mock Event Definitions for Testing ---

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct UserCreatedEvent {
        user_id: String,
        username: String,
    }

    impl Event for UserCreatedEvent {
        fn event_type() -> EventType {
            "UserCreatedEvent".to_string()
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct OrderPlacedEvent {
        order_id: Uuid,
        product_ids: Vec<u32>,
    }

    impl Event for OrderPlacedEvent {
        fn event_type() -> EventType {
            "OrderPlacedEvent".to_string()
        }
    }

    // A mock event that we won't register, to test unknown event handling
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct UnregisteredEvent {
        data: String,
    }

    impl Event for UnregisteredEvent {
        fn event_type() -> EventType {
            "UnregisteredEvent".to_string()
        }
    }

    // --- Tests for EventPackage ---

    #[test]
    fn test_event_package_new() {
        let event = UserCreatedEvent {
            user_id: "A1".to_string(),
            username: "Avenger-1".to_string(),
        };
        let source = MicroService::Train;

        let package = EventPackage::new(source, event.clone());

        assert_eq!(package.source, source);
        assert_eq!(package.name, UserCreatedEvent::event_type());

        // Check if the payload can be deserialized back to the original event
        let deserialized_event: UserCreatedEvent = serde_json::from_value(package.event).unwrap();
        assert_eq!(deserialized_event, event);
    }

    #[test]
    fn test_event_package_topic_key() {
        let event = OrderPlacedEvent {
            order_id: Uuid::new_v4(),
            product_ids: vec![101, 202],
        };
        let source = MicroService::Train;
        let package = EventPackage::new(source, event);

        let expected_key = format!("{}.{}", MicroService::Train, OrderPlacedEvent::event_type());
        assert_eq!(package.topic_key(), expected_key);
    }

    // --- Tests for EventRegistry ---

    #[test]
    fn test_event_registry_register_and_deserialize_success() {
        let mut registry = EventRegistry::new();

        // Register the event types we expect to handle
        registry.register::<UserCreatedEvent>();
        registry.register::<OrderPlacedEvent>();

        // 1. Create a package for UserCreatedEvent
        let user_event = UserCreatedEvent {
            user_id: "user-456".to_string(),
            username: "Bob".to_string(),
        };
        let user_package = EventPackage::new(MicroService::Train, user_event.clone());

        // 2. Deserialize it using the registry
        let deserialized_result = registry.deserialize(&user_package).unwrap();
        let any_box = deserialized_result.unwrap();

        // 3. Downcast the `Box<dyn Any>` back to the concrete type
        let downcasted_event = any_box.downcast_ref::<UserCreatedEvent>();

        // 4. Assert that downcasting was successful and data is correct
        assert!(downcasted_event.is_some());
        assert_eq!(downcasted_event.unwrap(), &user_event);
    }

    #[test]
    fn test_event_registry_deserialize_unregistered_event() {
        let mut registry = EventRegistry::new();
        // We only register UserCreatedEvent
        registry.register::<UserCreatedEvent>();

        // Create a package for an event type that is NOT registered
        let unregistered_event = UnregisteredEvent {
            data: "some data".to_string(),
        };
        let package = EventPackage::new(MicroService::Train, unregistered_event);

        // The deserialize method should return None because the event type is unknown
        let result = registry.deserialize(&package);
        assert!(result.is_none());
    }

    #[test]
    fn test_event_registry_deserialize_payload_mismatch() {
        let mut registry = EventRegistry::new();
        registry.register::<UserCreatedEvent>();

        // Create a package where the `name` field indicates a UserCreatedEvent,
        // but the `event` payload is actually from an OrderPlacedEvent.
        // This simulates a corrupted or mismatched message.
        let order_event_payload = serde_json::to_value(OrderPlacedEvent {
            order_id: Uuid::new_v4(),
            product_ids: vec![1, 2],
        })
        .unwrap();

        let mismatched_package = EventPackage {
            id: Uuid::new_v4(),
            source: MicroService::Train,
            time: Utc::now(),
            name: UserCreatedEvent::event_type(), // Name says it's a UserCreatedEvent...
            event: order_event_payload,           // ...but payload is something else
        };

        // The registry should find the deserializer for UserCreatedEvent,
        // but the deserializer itself should fail because the JSON structure doesn't match.
        let result = registry.deserialize(&mismatched_package);

        // We expect `Some(Err(...))`
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }
}

/// 城市数据已更新。
///
/// 触发位置:
/// - `load_city`: `base/src/infrastructure/application/service/train_data.rs:165`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityUpdatedEvent;

impl Event for CityUpdatedEvent {
    fn event_type() -> EventType {
        "city_updated_event".to_string()
    }
}

/// 车站数据已更新。
///
/// 触发位置:
/// - `load_station`: `base/src/infrastructure/application/service/train_data.rs:188`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationUpdatedEvent;

impl Event for StationUpdatedEvent {
    fn event_type() -> EventType {
        "station_updated_event".to_string()
    }
}

/// 用户信息（个人资料、密码等）已更新。
///
/// 触发位置:
/// - `set_profile`: `base/src/infrastructure/application/service/user_profile.rs:162`
/// - `register`: `base/src/infrastructure/application/service/user_manager.rs:101`
/// - `update_password`: `base/src/infrastructure/application/service/user_manager.rs:215`
/// - `set_payment_password`: `base/src/infrastructure/application/service/transaction.rs:211`
///
/// 注意：分析时发现一个潜在问题，`wrong_payment_password_tried` 的值似乎并未在相关操作中被更新。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUpdatedEvent;

impl Event for UserUpdatedEvent {
    fn event_type() -> EventType {
        "user_updated_event".to_string()
    }
}

/// 用户的个人身份信息已更新。
///
/// 触发位置:
/// - `set_personal_info`: `base/src/infrastructure/application/service/personal_info.rs:140`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalInfoUpdatedEvent;

impl Event for PersonalInfoUpdatedEvent {
    fn event_type() -> EventType {
        "personal_info_updated_event".to_string()
    }
}

/// 列车基础信息已更新。
///
/// 触发位置:
/// - `save_raw_train_number`: `base/src/infrastructure/repository/train.rs:953`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainUpdatedEvent;

impl Event for TrainUpdatedEvent {
    fn event_type() -> EventType {
        "train_updated_event".to_string()
    }
}

/// 列车时刻表已更新。
///
/// 触发位置:
/// - `auto_plan_schedule`: `base/src/infrastructure/service/train_schedule.rs:288`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainScheduleUpdatedEvent;

impl Event for TrainScheduleUpdatedEvent {
    fn event_type() -> EventType {
        "train_schedule_updated_event".to_string()
    }
}

/// 路线信息已更新。
///
/// 触发位置:
/// - `save_raw_train_number`: `base/src/infrastructure/repository/train.rs:953`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteUpdatedEvent;

impl Event for RouteUpdatedEvent {
    fn event_type() -> EventType {
        "route_updated_event".to_string()
    }
}

/// 座位类型或其映射关系已更新。
///
/// 触发位置:
/// - `load_train_type`: `base/src/infrastructure/application/service/train_data.rs:218`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatTypeUpdatedEvent;

impl Event for SeatTypeUpdatedEvent {
    fn event_type() -> EventType {
        "seat_type_updated_event".to_string()
    }
}

/// 酒店信息（包括评论、订单处理等）已更新。
///
/// 触发位置:
/// - `load_hotel`: `base/src/infrastructure/application/service/hotel_data.rs:62`
/// - `new_comment`: `base/src/infrastructure/application/service/hotel.rs:126`
/// - `process_hotel_orders`: `base/src/infrastructure/application/service/hotel_order.rs:336`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotelUpdatedEvent;

impl Event for HotelUpdatedEvent {
    fn event_type() -> EventType {
        "hotel_updated_event".to_string()
    }
}

/// 酒店房型信息已更新。
///
/// 触发位置:
/// - `load_hotel`: `base/src/infrastructure/application/service/hotel_data.rs:62`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotelRoomTypeUpdatedEvent;

impl Event for HotelRoomTypeUpdatedEvent {
    fn event_type() -> EventType {
        "hotel_room_type_updated_event".to_string()
    }
}

/// 火车餐菜品信息已更新。
///
/// 触发位置:
/// - `load_dish_takeaway`: `base/src/infrastructure/application/service/train_data.rs:263`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DishUpdatedEvent;

impl Event for DishUpdatedEvent {
    fn event_type() -> EventType {
        "dish_updated_event".to_string()
    }
}

/// 外卖菜品信息已更新。
///
/// 触发位置:
/// - `load_dish_takeaway`: `base/src/infrastructure/application/service/train_data.rs:263`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TakeawayDishUpdatedEvent;

impl Event for TakeawayDishUpdatedEvent {
    fn event_type() -> EventType {
        "takeaway_dish_updated_event".to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TakeawayShopUpdatedEvent;

impl Event for TakeawayShopUpdatedEvent {
    fn event_type() -> EventType {
        "takeaway_shop_updated_event".to_string()
    }
}
