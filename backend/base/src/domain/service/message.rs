use crate::domain::model::message::Notify;
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessageServiceError {
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid user id: {0}")]
    InvalidUserId(UserId),
}

#[async_trait]
pub trait MessageService: 'static + Send + Sync {
    async fn send_to_user(
        &self,
        user_id: UserId,
        message: Box<dyn Notify>,
    ) -> Result<(), MessageServiceError>;

    async fn get_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Box<dyn Notify>>, MessageServiceError>;
}
