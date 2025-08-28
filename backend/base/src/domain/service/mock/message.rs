#![cfg(test)]

use crate::application::service::message::NotifyDTO;
use crate::domain::model::message::Notify;
use crate::domain::model::user::UserId;
use async_trait::async_trait;
use mockall::mock;

use crate::domain::service::message::{MessageService, MessageServiceError};

mock! {
    pub MessageService {}

    #[async_trait]
    impl MessageService for MessageService {
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
}

pub fn message_service_mock() -> Box<dyn MessageService> {
    Box::new(MockMessageService::new())
}
