use crate::domain::model::order::OrderType;
use crate::domain::service::order_status::{OrderStatusMessage, OrderStatusMessagePack};
use crate::infrastructure::RABBITMQ_ORDER_STATUS_EXCHANGE_NAME;
use lapin::options::{BasicPublishOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, ConnectionProperties, ExchangeKind};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, instrument};

pub struct OrderStatusProducerService {
    channel: Arc<lapin::Channel>,
}

#[derive(Debug, Error)]
pub enum OrderStatusProducerServiceError {
    #[error("connection error: {0}")]
    ConnectionError(lapin::Error),
}

impl OrderStatusProducerService {
    pub async fn new(connection_string: &str) -> Result<Self, OrderStatusProducerServiceError> {
        let connection =
            lapin::Connection::connect(connection_string, ConnectionProperties::default())
                .await
                .map_err(OrderStatusProducerServiceError::ConnectionError)?;

        let channel = connection
            .create_channel()
            .await
            .map_err(OrderStatusProducerServiceError::ConnectionError)?;

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
            .map_err(OrderStatusProducerServiceError::ConnectionError)?;

        Ok(OrderStatusProducerService {
            channel: Arc::new(channel),
        })
    }

    #[instrument(skip(self))]
    pub async fn delivery_message(
        &self,
        messages: OrderStatusMessagePack,
    ) -> Result<(), OrderStatusProducerServiceError> {
        let mut messages_by_type: HashMap<OrderType, Vec<OrderStatusMessage>> = HashMap::new();

        for message in messages.messages {
            messages_by_type
                .entry(message.order_type)
                .or_default()
                .push(message);
        }

        for (message_type, message_list) in messages_by_type {
            let new_pack = OrderStatusMessagePack {
                transaction_uuid: messages.transaction_uuid,
                atomic: messages.atomic,
                messages: message_list,
            };

            let payload = serde_json::to_vec(&new_pack).unwrap();

            info!(
                "Publishing message type {} with exchange {}, routing key {}",
                message_type,
                RABBITMQ_ORDER_STATUS_EXCHANGE_NAME,
                message_type.message_queue_name()
            );

            self.channel
                .basic_publish(
                    RABBITMQ_ORDER_STATUS_EXCHANGE_NAME,
                    message_type.message_queue_name(),
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default(),
                )
                .await
                .inspect_err(|e| error!("Failed to publish message: {}", e))
                .map_err(OrderStatusProducerServiceError::ConnectionError)?;
        }

        Ok(())
    }
}
