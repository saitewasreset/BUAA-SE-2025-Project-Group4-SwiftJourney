use crate::domain::service::order_status::OrderStatusMessagePack;
use crate::infrastructure::RABBITMQ_ORDER_STATUS_EXCHANGE_NAME;
use crate::infrastructure::messaging::consumer::order_status::RabbitMQOrderStatusConsumer;
use lapin::options::{
    BasicConsumeOptions, BasicNackOptions, ExchangeDeclareOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{ConnectionProperties, ExchangeKind};
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tracing::{error, instrument};

pub struct OrderStatusConsumerService {
    consumer_handler: Vec<JoinHandle<()>>,
}

#[instrument(skip_all)]
async fn consume_task(channel: lapin::Channel, consumer: Box<dyn RabbitMQOrderStatusConsumer>) {
    let queue_options = QueueDeclareOptions {
        passive: false,
        durable: true,
        exclusive: false,
        auto_delete: false,
        nowait: false,
    };
    let _ = channel
        .queue_declare(consumer.binding_key(), queue_options, FieldTable::default())
        .await
        .map_err(|e| error!("Failed to declare queue: {}", e))
        .unwrap();

    let consume_options = BasicConsumeOptions {
        no_local: false,
        no_ack: false,
        exclusive: false,
        nowait: false,
    };

    let mut c = channel
        .basic_consume(
            consumer.binding_key(),
            "",
            consume_options,
            FieldTable::default(),
        )
        .await
        .map_err(|e| error!("Failed to declare consumer: {}", e))
        .unwrap();

    while let Some(delivery) = c.next().await {
        match delivery {
            Ok(delivery) => {
                match serde_json::from_slice::<OrderStatusMessagePack>(&delivery.data) {
                    Ok(order_status) => {
                        if let Err(e) = consumer.consume(order_status).await {
                            error!("Failed to consume message: {}", e);

                            if let Err(e) = delivery.nack(BasicNackOptions::default()).await {
                                error!("Failed to nack message: {}", e);
                            }
                        } else if let Err(e) = delivery
                            .ack(lapin::options::BasicAckOptions::default())
                            .await
                        {
                            error!("Failed to ack message: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize message: {}", e);

                        if let Err(e) = delivery.nack(BasicNackOptions::default()).await {
                            error!("Failed to nack message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("failed to receive delivery: {}", e);
            }
        }
    }
}

impl OrderStatusConsumerService {
    pub async fn start(
        connection_string: &str,
        consumers: Vec<Box<dyn RabbitMQOrderStatusConsumer>>,
    ) -> Result<Self, OrderStatusConsumerServiceError> {
        let connection =
            lapin::Connection::connect(connection_string, ConnectionProperties::default())
                .await
                .map_err(OrderStatusConsumerServiceError::ConnectionError)?;

        let channel = connection
            .create_channel()
            .await
            .map_err(OrderStatusConsumerServiceError::ConnectionError)?;

        let exchange_options = ExchangeDeclareOptions {
            passive: false,
            durable: true,
            auto_delete: false,
            nowait: false,
            internal: false,
        };

        channel
            .exchange_declare(
                RABBITMQ_ORDER_STATUS_EXCHANGE_NAME,
                ExchangeKind::Direct,
                exchange_options,
                FieldTable::default(),
            )
            .await
            .map_err(OrderStatusConsumerServiceError::ConnectionError)?;

        let mut handlers = Vec::with_capacity(consumers.len());

        for consumer in consumers {
            let channel = connection
                .create_channel()
                .await
                .map_err(OrderStatusConsumerServiceError::ConnectionError)?;

            let handler = tokio::spawn(consume_task(channel, consumer));

            handlers.push(handler);
        }

        Ok(Self {
            consumer_handler: handlers,
        })
    }

    pub fn stop(&self) {
        for handler in &self.consumer_handler {
            handler.abort()
        }
    }
}

#[derive(Debug, Error)]
pub enum OrderStatusConsumerServiceError {
    #[error("connection error: {0}")]
    ConnectionError(lapin::Error),
}
