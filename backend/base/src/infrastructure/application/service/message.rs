use crate::application::commands::message::HistoryMessageQuery;
use crate::application::service::message::{
    MessageApplicationService, MessageApplicationServiceError, NotifyDTO,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::message::TripNotify;
use crate::domain::model::session::SessionId;
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use crate::domain::service::message::MessageService;
use crate::domain::service::order::OrderService;
use crate::domain::service::session::SessionManagerService;
use anyhow::anyhow;
use async_trait::async_trait;
use std::any::TypeId;
use std::sync::Arc;
use tracing::error;

pub struct MessageApplicationServiceImpl<OS, MS, SMS>
where
    OS: OrderService,
    MS: MessageService,
    SMS: SessionManagerService,
{
    order_service: Arc<OS>,
    message_service: Arc<MS>,
    session_manager_service: Arc<SMS>,
}

impl<OS, MS, SMS> MessageApplicationServiceImpl<OS, MS, SMS>
where
    OS: OrderService,
    MS: MessageService,
    SMS: SessionManagerService,
{
    pub fn new(
        order_service: Arc<OS>,
        message_service: Arc<MS>,
        session_manager_service: Arc<SMS>,
    ) -> Self {
        MessageApplicationServiceImpl {
            order_service,
            message_service,
            session_manager_service,
        }
    }
}

#[async_trait]
impl<OS, MS, SMS> MessageApplicationService for MessageApplicationServiceImpl<OS, MS, SMS>
where
    OS: OrderService,
    MS: MessageService,
    SMS: SessionManagerService,
{
    async fn get_history(
        &self,
        query: HistoryMessageQuery,
    ) -> Result<Vec<NotifyDTO>, Box<dyn ApplicationError>> {
        let session_id =
            SessionId::try_from(query.session_id.as_ref()).map_err(|_for_super_earth| {
                GeneralError::BadRequest(format!("invalid session id format: {}", query.session_id))
            })?;

        let user_id = self
            .session_manager_service
            .get_user_id_by_session(session_id)
            .await
            .inspect_err(|e| {
                error!("Failed to get user ID by session: {:?}", e);
            })
            .map_err(|e| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

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
