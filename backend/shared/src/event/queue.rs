//! 提供了基于 RabbitMQ 的事件发布和订阅服务。
//!
//! 这个模块定义了 `EventService` trait，它抽象了微服务与事件队列交互所需的
//! 核心功能，包括初始化消费者、发布事件和处理接收到的事件。

use crate::MicroService;
use crate::event::{EventPackage, EventRegistry, RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME};
use async_trait::async_trait;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties, ExchangeKind};
use std::any::Any;
use std::sync::{Arc, Mutex};
use tokio_stream::StreamExt;
use tracing::{error, info};

/// `EventService` 相关的错误类型。
#[derive(Debug)]
pub enum EventServiceError {
    /// 包装了来自底层队列服务（如 `lapin` 或 `anyhow`）的错误。
    QueueServiceError(anyhow::Error),
}

/// 创建并返回一个到 RabbitMQ 的 `lapin::Channel`。
///
/// # Arguments
/// * `addr`: RabbitMQ 的连接地址，例如 "amqp://A1:Avenger1@terra:5672/%2f"。
pub async fn get_channel(addr: &str) -> Result<lapin::Channel, EventServiceError> {
    let conn = Connection::connect(addr, ConnectionProperties::default())
        .await
        .inspect_err(|e| error!("Failed to connect to rabbitmq: {:?} addr = {}", e, addr))
        .map_err(|e| EventServiceError::QueueServiceError(e.into()))?;

    conn.create_channel()
        .await
        .inspect_err(|e| error!("Failed to create rabbitmq channel: {:?} addr = {}", e, addr))
        .map_err(|e| EventServiceError::QueueServiceError(e.into()))
}

/// `EventService` trait 定义了微服务事件处理的核心逻辑。
///
/// 任何需要收发领域事件的微服务都应该实现这个 trait。
/// 它封装了与 RabbitMQ 的所有交互细节。
#[async_trait]
pub trait EventService: 'static + Send + Sync {
    /// 返回当前微服务的标识。
    fn micro_service(&self) -> MicroService;
    /// 返回用于与 RabbitMQ 通信的 `lapin::Channel`。
    fn lapin_channel(&self) -> lapin::Channel;
    /// 返回一个自身的 `Arc` 引用，用于在异步任务中共享服务实例。
    fn self_arc(&self) -> Arc<Self>;
    /// 返回一个包裹在 `Arc<Mutex<...>>` 中的 `EventRegistry` 实例。
    /// 使用 `Mutex` 确保在多线程环境下对注册中心的访问是安全的。
    fn event_registry(&self) -> Arc<Mutex<EventRegistry>>;

    /// 初始化 RabbitMQ 消费者。
    ///
    /// 这个方法会执行以下操作：
    /// 1. 声明一个 Topic Exchange。
    /// 2. 声明一个与当前微服务同名的持久化队列。
    /// 3. 将队列绑定到 Exchange，并订阅所有主题 (`#`)。
    /// 4. 启动一个后台异步任务 (`tokio::spawn`)，持续监听队列中的消息。
    /// 5. 在接收到消息后，反序列化为 `EventPackage`，再通过 `EventRegistry`
    ///    反序列化为具体的事件 `Box<dyn Any>`，最后调用 `handle_event` 方法进行处理。
    ///
    /// **注意**: 默认绑定路由键为 `#`，表示会接收所有事件。
    async fn init_consumer(&self) -> Result<(), EventServiceError> {
        let channel = self.lapin_channel();

        channel
            .exchange_declare(
                RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to declare exchange {:?} exchange: {}",
                    e, RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME
                )
            })
            .map_err(|e| EventServiceError::QueueServiceError(e.into()))?;

        let queue_name = self.micro_service().to_string();

        channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .inspect_err(|e| error!("Failed to declare queue: {:?} queue: {}", e, queue_name))
            .map_err(|e| EventServiceError::QueueServiceError(e.into()))?;

        channel
            .queue_bind(
                &queue_name,
                RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME,
                "#",
                QueueBindOptions { nowait: false },
                FieldTable::default(),
            )
            .await
            .inspect_err(|e| error!("Failed to bind queue: {:?} queue: {}", e, queue_name))
            .map_err(|e| EventServiceError::QueueServiceError(e.into()))?;

        let consumer_tag = format!("{}_consumer", self.micro_service());

        let mut consumer = channel
            .basic_consume(
                &queue_name,
                &consumer_tag,
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to declare consumer: {:?} consumer: {}",
                    e, consumer_tag
                )
            })
            .map_err(|e| EventServiceError::QueueServiceError(e.into()))?;

        let self_arc = self.self_arc();
        let event_registry = self.event_registry();

        let consumer_callback = async move {
            while let Some(delivery) = consumer.next().await {
                match delivery {
                    Ok(delivery) => {
                        info!(
                            "Received message, routing key = {}, delivery tag = {}",
                            delivery.routing_key, delivery.delivery_tag
                        );

                        match serde_json::from_slice::<EventPackage>(&delivery.data) {
                            Ok(event_package) => {
                                info!("Event package parsed: {}", event_package);

                                let deserialize_result =
                                    { event_registry.lock().unwrap().deserialize(&event_package) };

                                match deserialize_result {
                                    Some(Ok(event)) => {
                                        if let Err(e) = self_arc.handle_event(event).await {
                                            error!("Failed to handle event: {:?}", e);
                                        }
                                    }
                                    Some(Err(e)) => {
                                        error!("Failed to deserialize event: {:?}", e);
                                    }
                                    None => {
                                        error!("Unknown event: {:?}", event_package);
                                    }
                                }
                            }
                            Err(err) => error!("Failed to parse event package: {:?}", err),
                        }

                        if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                            error!("Failed to acknowledge message: {:?}", e);
                        }
                    }
                    Err(e) => error!("Delivery error: {:?}", e),
                }
            }
        };

        tokio::spawn(consumer_callback);

        Ok(())
    }

    /// 发布一个事件到 RabbitMQ。
    /// * `event_package`: 要发布的 `EventPackage`。事件将使用 `topic_key` 作为路由键
    ///   被发送到 Topic Exchange。
    async fn publish_event(&self, event_package: EventPackage) -> Result<(), EventServiceError> {
        let channel = self.lapin_channel();

        let routing_key = event_package.topic_key();

        let payload = serde_json::to_vec(&event_package).unwrap();

        channel
            .basic_publish(
                RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME,
                &routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .inspect_err(|e| error!("Failed to publish event: {} event: {:?}", e, event_package))
            .map_err(|e| EventServiceError::QueueServiceError(e.into()))?;

        Ok(())
    }

    /// 处理一个已成功反序列化的事件。
    ///
    /// 这是 `EventService` 实现者必须提供的核心业务逻辑。
    /// 方法接收一个类型被擦除的事件 `Box<dyn Any>`，实现者需要使用 `downcast_ref` 或
    /// 类似的 `Any` 方法将其转换回具体的事件类型，然后执行相应的处理逻辑。
    ///
    /// # Arguments
    /// * `event`: 一个包含了具体领域事件的 `Box<dyn Any>`。
    ///
    /// # Returns
    /// - `Ok(())`: 如果事件处理成功。
    /// - `Err(EventServiceError)`: 如果处理过程中发生错误。
    ///
    /// # Example
    /// ```ignore
    /// // In the implementation of a specific service
    /// async fn handle_event(&self, event: Box<dyn Any + Sync + Send>) -> Result<(), EventServiceError> {
    ///     if let Some(user_created) = event.downcast_ref::<UserCreatedEvent>() {
    ///         println!("Handling UserCreatedEvent for user: {}", user_created.username);
    ///         // ... business logic for user creation ...
    ///     } else if let Some(order_placed) = event.downcast_ref::<OrderPlacedEvent>() {
    ///         println!("Handling OrderPlacedEvent for order: {}", order_placed.order_id);
    ///         // ... business logic for order placement ...
    ///     } else {
    ///         println!("Received an unknown event type.");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    async fn handle_event(
        &self,
        event: Box<dyn Any + Send + Sync>,
    ) -> Result<(), EventServiceError>;
}
