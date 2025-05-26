use crate::application::ApplicationError;
use crate::application::commands::message::HistoryMessageQuery;
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use crate::domain::service::order::order_dto::OrderInfoDto;
use async_trait::async_trait;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize)]
pub struct Message<T: Serialize> {
    #[serde(rename = "type")]
    pub type_name: String,
    pub data: T,
}

#[derive(Serialize, Clone)]
pub enum NotifyDTO {
    Order(OrderNotifyDTO),
    Trip(TripNotifyDTO),
}

#[derive(Serialize, Clone)]
pub struct OrderNotifyDTO {
    pub title: String,
    pub message_time: DateTimeWithTimeZone,
    pub order: Box<OrderInfoDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TripNotifyDTO {
    pub title: String,
    pub message_time: DateTimeWithTimeZone,
    pub train_number: String,
    pub departure_time: DateTimeWithTimeZone,
    pub departure_station: String,
    pub arrival_station: String,
}

#[derive(Debug, Error)]
pub enum MessageApplicationServiceError {
    #[error("an infrastructure error occurred")]
    InfrastructureError(ServiceError),
}

impl From<NotifyDTO> for Message<NotifyDTO> {
    fn from(notify: NotifyDTO) -> Self {
        Message {
            type_name: match notify {
                NotifyDTO::Order(_) => "order".to_string(),
                NotifyDTO::Trip(_) => "trip".to_string(),
            },
            data: notify,
        }
    }
}

impl ApplicationError for MessageApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            MessageApplicationServiceError::InfrastructureError(_) => 500, // Internal Server Error
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[async_trait]
pub trait MessageApplicationService: 'static + Send + Sync {
    async fn get_history(
        &self,
        query: HistoryMessageQuery,
    ) -> Result<Vec<NotifyDTO>, Box<dyn ApplicationError>>;
}
