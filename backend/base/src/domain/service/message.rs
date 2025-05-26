use crate::application::service::message::{Message, MessageApplicationServiceError, NotifyDTO};
use crate::domain::model::message::Notify;
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use dyn_clone::DynClone;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessageServiceError {
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid user id: {0}")]
    InvalidUserId(UserId),
    #[error("session closed: {0}")]
    SessionClosed(anyhow::Error),
}

#[async_trait]
pub trait MessageListener: 'static + Send + Sync + DynClone {
    async fn check_session(&mut self) -> bool;

    async fn on_message(&mut self, message: Vec<u8>) -> bool;
}

dyn_clone::clone_trait_object!(MessageListener);

#[async_trait]
pub trait MessageListenerService: 'static + Send + Sync {
    fn add_listener(&self, user_id: UserId, listener: Box<dyn MessageListener>);

    fn find_listener_by_user_id(&self, user_id: UserId) -> Vec<Box<dyn MessageListener>>;

    async fn check_session(&self);
}

#[async_trait]
pub trait MessageService: 'static + Send + Sync {
    async fn convert_notify_to_dto(
        &self,
        notify: Box<dyn Notify>,
    ) -> Result<NotifyDTO, MessageServiceError>;

    async fn send_to_user(
        &self,
        user_id: UserId,
        notify: Box<dyn Notify>,
    ) -> Result<(), MessageServiceError>;

    async fn get_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Box<dyn Notify>>, MessageServiceError>;
}
