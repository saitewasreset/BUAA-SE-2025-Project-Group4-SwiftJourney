use crate::domain::model::message::Notify;
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use crate::domain::service::order::order_dto::OrderInfoDto;
use async_trait::async_trait;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize)]
pub enum NotifyDTO {
    Order(OrderNotifyDTO),
    Trip(TripNotifyDTO),
}

#[derive(Serialize)]
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
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
}

#[async_trait]
pub trait MessageApplicationService: 'static + Send + Sync {
    async fn convert_notify_to_dto(
        &self,
        notify: Box<dyn Notify>,
    ) -> Result<NotifyDTO, MessageApplicationServiceError>;

    async fn get_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<NotifyDTO>, MessageApplicationServiceError>;
}
