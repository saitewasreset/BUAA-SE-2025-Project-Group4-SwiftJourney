use crate::application::service::message::{
    MessageApplicationService, MessageApplicationServiceError, NotifyDTO,
};
use crate::domain::model::message::TripNotify;
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use crate::domain::service::message::MessageService;
use crate::domain::service::order::OrderService;
use anyhow::anyhow;
use async_trait::async_trait;
use std::any::TypeId;
use std::sync::Arc;
use tracing::error;

pub struct MessageApplicationServiceImpl<OS, MS>
where
    OS: OrderService,
    MS: MessageService,
{
    order_service: Arc<OS>,
    message_service: Arc<MS>,
}

impl<OS, MS> MessageApplicationServiceImpl<OS, MS>
where
    OS: OrderService,
    MS: MessageService,
{
    pub fn new(order_service: Arc<OS>, message_service: Arc<MS>) -> Self {
        MessageApplicationServiceImpl {
            order_service,
            message_service,
        }
    }
}

#[async_trait]
impl<OS, MS> MessageApplicationService for MessageApplicationServiceImpl<OS, MS>
where
    OS: OrderService,
    MS: MessageService,
{
    async fn get_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<NotifyDTO>, MessageApplicationServiceError> {
        let notify_list = self
            .message_service
            .get_history(user_id)
            .await
            .inspect_err(|e| {
                error!("Failed to get message history: {:?}", e);
            })
            .map_err(|e| {
                MessageApplicationServiceError::InfrastructureError(
                    ServiceError::RelatedServiceError(anyhow!("message service error: {}", e)),
                )
            })?;

        let mut notify_dto_list = Vec::new();

        for notify in notify_list {
            let notify_dto = self
                .message_service
                .convert_notify_to_dto(notify)
                .await
                .inspect_err(|e| {
                    error!("Failed to convert notify to DTO: {:?}", e);
                })
                .map_err(|e| {
                    MessageApplicationServiceError::InfrastructureError(
                        ServiceError::RelatedServiceError(anyhow!("message service error: {}", e)),
                    )
                })?;
            notify_dto_list.push(notify_dto);
        }

        Ok(notify_dto_list)
    }
}
