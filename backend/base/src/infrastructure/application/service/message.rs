use crate::application::commands::message::HistoryMessageQuery;
use crate::application::service::message::{
    MessageApplicationService, MessageApplicationServiceError, NotifyDTO,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::session::SessionId;
use crate::domain::service::ServiceError;
use crate::domain::service::message::MessageService;
use crate::domain::service::session::SessionManagerService;
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::error;

pub struct MessageApplicationServiceImpl<MS, SMS>
where
    MS: MessageService,
    SMS: SessionManagerService,
{
    message_service: Arc<MS>,
    session_manager_service: Arc<SMS>,
}

impl<MS, SMS> MessageApplicationServiceImpl<MS, SMS>
where
    MS: MessageService,
    SMS: SessionManagerService,
{
    pub fn new(message_service: Arc<MS>, session_manager_service: Arc<SMS>) -> Self {
        MessageApplicationServiceImpl {
            message_service,
            session_manager_service,
        }
    }
}

#[async_trait]
impl<MS, SMS> MessageApplicationService for MessageApplicationServiceImpl<MS, SMS>
where
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
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::domain::service::mock::message::MockMessageService;
    use crate::domain::service::mock::session::MockSessionManagerService;
    use crate::application::commands::message::HistoryMessageQuery;
    use crate::domain::model::user::UserId;
    use crate::application::service::message::{NotifyDTO, OrderNotifyDTO};
    use crate::domain::service::order::order_dto::{BaseOrderDto, DishOrderDto};
    use crate::domain::service::order::order_dto::OrderInfoDto::Dish;
    use crate::domain::service::message::MessageServiceError;
    use crate::domain::model::message::{Notify, NotifyType};
    use crate::domain::model::mock::message::MockNotify;

    // ================= get_history =================
    #[tokio::test]
    async fn test_get_history_success() {
        // Mock session service
        let mut session_service = MockSessionManagerService::new();
        session_service
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(UserId::from(1))));

        // Mock message service
        let mut message_service = MockMessageService::new();
        message_service
            .expect_get_history()
            .returning(|_| {
                Ok(vec![Box::new(MockNotify::new(1u64.into(), "title order", NotifyType::Order)) as Box<dyn Notify>])
            });
        message_service
            .expect_convert_notify_to_dto()
            .returning(|_| Ok(NotifyDTO::Order(OrderNotifyDTO {
                title: "title order".to_string(),
                message_time: Default::default(),
                order: Box::new(Dish(DishOrderDto {
                    base: BaseOrderDto {
                        order_id: "1".to_string(),
                        status: "完成".to_string(),
                        unit_price: 0.0,
                        amount: 0,
                        can_cancel: false,
                        reason: None,
                        order_type: "".to_string(),
                    },
                    train_number: "G123".to_string(),
                    departure_time: "10:00".to_string(),
                    dish_time: "lunch".to_string(),
                    name: "日升".to_string(),
                    dish_name: "鱼".to_string(),
                })),
            })));

        let service = MessageApplicationServiceImpl::new(
            Arc::new(message_service),
            Arc::new(session_service),
        );

        let query = HistoryMessageQuery {
            session_id: uuid::Uuid::new_v4().into(),
            limit: None,
        };

        let result = service.get_history(query).await.unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            NotifyDTO::Order(order) => {
                assert_eq!(order.title, "title order".to_string());
                assert!(matches!(order.order.as_ref(), Dish(x) if x.train_number == "G123"));
            }
            _ => panic!("Expected Order variant"),
        }
    }


    #[tokio::test]
    async fn test_get_history_invalid_session() {
        let mut session_service = MockSessionManagerService::new();
        session_service
            .expect_get_user_id_by_session()
            .returning(|_| Ok(None)); // 模拟无效 session

        let message_service = MockMessageService::new();

        let service = MessageApplicationServiceImpl::new(
            Arc::new(message_service),
            Arc::new(session_service),
        );

        let query = HistoryMessageQuery {
            session_id: uuid::Uuid::new_v4().into(),
            limit: None,
        };

        let result = service.get_history(query).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_history_message_service_error() {
        let mut session_service = MockSessionManagerService::new();
        session_service
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(UserId::from(1))));

        let mut message_service = MockMessageService::new();
        message_service
            .expect_get_history()
            .returning(|_| Err(MessageServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!("message service error"))
            )));

        let service = MessageApplicationServiceImpl::new(
            Arc::new(message_service),
            Arc::new(session_service),
        );

        let query = HistoryMessageQuery {
            session_id: uuid::Uuid::new_v4().into(),
            limit: None,
        };

        let result = service.get_history(query).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_history_convert_notify_error() {
        let mut session_service = MockSessionManagerService::new();
        session_service
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(UserId::from(1))));

        let mut message_service = MockMessageService::new();
        message_service
            .expect_get_history()
            .returning(|_| {
                Ok(vec![Box::new(MockNotify::new(1u64.into(), "title order", NotifyType::Order)) as Box<dyn Notify>])
            });
        message_service
            .expect_convert_notify_to_dto()
            .returning(|_| Err(MessageServiceError::InfrastructureError(
                ServiceError::RelatedServiceError(anyhow!("message service error"))
            )));

        let service = MessageApplicationServiceImpl::new(
            Arc::new(message_service),
            Arc::new(session_service),
        );

        let query = HistoryMessageQuery {
            session_id: uuid::Uuid::new_v4().into(),
            limit: None,
        };

        let result = service.get_history(query).await;
        assert!(result.is_err());
    }
}
