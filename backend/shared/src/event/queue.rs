use crate::MicroService;
use crate::event::{Event, EventPackage, RABBITMQ_EVENT_QUEUE_EXCHANGE_NAME};
use async_trait::async_trait;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties, ExchangeKind};
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::{error, info};

#[derive(Debug)]
pub enum EventServiceError {
    QueueServiceError(anyhow::Error),
}

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

#[async_trait]
pub trait EventService: 'static + Send + Sync {
    fn micro_service(&self) -> MicroService;
    fn lapin_channel(&self) -> lapin::Channel;
    fn self_arc(&self) -> Arc<Self>;

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

                                if let Err(e) = self_arc.handle_event(event_package.event).await {
                                    error!("Failed to handle event: {:?}", e);
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

    async fn handle_event(&self, event: Event) -> Result<(), EventServiceError>;
}
