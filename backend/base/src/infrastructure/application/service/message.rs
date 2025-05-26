use crate::application::service::message::{
    MessageApplicationService, MessageApplicationServiceError, NotifyDTO, OrderNotifyDTO,
    TripNotifyDTO,
};
use crate::domain::model::message::{Notify, OrderNotify, TripNotify};
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use crate::domain::service::message::MessageService;
use crate::domain::service::order::OrderService;
use anyhow::anyhow;
use async_trait::async_trait;
use std::any::{Any, TypeId};
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
    async fn convert_notify_to_dto(
        &self,
        notify: Box<dyn Notify>,
    ) -> Result<NotifyDTO, MessageApplicationServiceError> {
        let type_id = (*notify).type_id();

        let notify_any = dyn_clone::clone_box(notify.as_ref()) as Box<dyn Any>;

        if type_id == TypeId::of::<OrderNotify>() {
            let order_notify = notify_any.downcast::<OrderNotify>().unwrap();

            let order_dto = self
                .order_service
                .convert_order_to_dto(dyn_clone::clone_box(order_notify.order()))
                .await
                .inspect_err(|e| {
                    error!("Failed to convert order to DTO: {:?}", e);
                })
                .map_err(|e| {
                    MessageApplicationServiceError::InfrastructureError(
                        ServiceError::RelatedServiceError(anyhow!("order service error: {}", e)),
                    )
                })?;

            Ok(NotifyDTO::Order(OrderNotifyDTO {
                title: order_notify.title().to_string(),
                message_time: order_notify.message_time(),
                order: Box::new(order_dto),
            }))
        } else if type_id == TypeId::of::<TripNotify>() {
            let trip_notify = notify_any.downcast::<TripNotify>().unwrap();

            Ok(NotifyDTO::Trip(TripNotifyDTO {
                title: trip_notify.title().to_string(),
                message_time: trip_notify.message_time(),
                train_number: trip_notify.train_number().to_string(),
                departure_time: trip_notify.departure_time(),
                departure_station: trip_notify.departure_station().to_string(),
                arrival_station: trip_notify.arrival_station().to_string(),
            }))
        } else {
            panic!("Unknown notify type");
        }
    }

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
            let notify_dto = self.convert_notify_to_dto(notify).await?;
            notify_dto_list.push(notify_dto);
        }

        Ok(notify_dto_list)
    }
}
