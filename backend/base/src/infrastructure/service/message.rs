use crate::application::service::message::{Message, NotifyDTO, OrderNotifyDTO, TripNotifyDTO};
use crate::domain::model::message::{Notify, OrderNotify, TripNotify};
use crate::domain::model::user::UserId;
use crate::domain::repository::notify::NotifyRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::message::{
    MessageListener, MessageListenerService, MessageService, MessageServiceError,
};
use crate::domain::service::order::OrderService;
use anyhow::anyhow;
use async_trait::async_trait;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{debug, error, instrument, trace};

#[derive(Clone)]
pub struct MessageListenerImpl {
    session: actix_ws::Session,
}

impl MessageListenerImpl {
    pub fn new(session: actix_ws::Session) -> Self {
        Self { session }
    }
}

#[async_trait]
impl MessageListener for MessageListenerImpl {
    async fn check_session(&mut self) -> bool {
        self.session
            .ping(b"For Super Earth!\nHelldivers never die!")
            .await
            .is_ok()
    }

    async fn on_message(&mut self, message: Vec<u8>) -> bool {
        self.session.binary(message).await.is_ok()
    }
}

pub struct MessageListenerServiceImpl {
    max_concurrent_session_per_user: usize,
    listeners: DashMap<UserId, VecDeque<Box<dyn MessageListener>>>,
}

impl MessageListenerServiceImpl {
    pub fn new(max_concurrent_session_per_user: usize) -> Self {
        Self {
            max_concurrent_session_per_user,
            listeners: DashMap::new(),
        }
    }
}

#[async_trait]
impl MessageListenerService for MessageListenerServiceImpl {
    #[instrument(skip(self, listener))]
    fn add_listener(&self, user_id: UserId, listener: Box<dyn MessageListener>) {
        trace!("Adding listener for user_id: {}", user_id);
        let mut user_session_list = self.listeners.entry(user_id).or_default();

        if user_session_list.len() == self.max_concurrent_session_per_user {
            debug!("Removing oldest listener for user_id: {}", user_id);
            user_session_list.pop_front();
        }

        user_session_list.push_front(listener);
    }

    #[instrument(skip(self))]
    fn find_listener_by_user_id(&self, user_id: UserId) -> Vec<Box<dyn MessageListener>> {
        trace!("Finding listeners for user_id: {}", user_id);

        Vec::from(
            self.listeners
                .get(&user_id)
                .map(|entry| entry.value().clone())
                .unwrap_or_default(),
        )
    }

    async fn check_session(&self) {
        // 第一步：收集所有需要检查的条目
        let user_ids: Vec<UserId> = self.listeners.iter().map(|entry| *entry.key()).collect();

        for user_id in user_ids {
            if let Some(mut entry) = self.listeners.get_mut(&user_id) {
                let user_session_list = entry.value_mut();

                // 检查每个 session 并重建列表
                let mut valid_sessions = VecDeque::new();

                // 由于需要异步操作，我们需要一个一个处理
                while let Some(mut session) = user_session_list.pop_front() {
                    if session.check_session().await {
                        valid_sessions.push_back(session);
                    }
                }

                if valid_sessions.is_empty() {
                    drop(entry); // 释放锁
                    self.listeners.remove(&user_id);
                } else {
                    *user_session_list = valid_sessions;
                }
            }
        }
    }
}

pub struct MessageServiceImpl<MLS, NR, OS>
where
    MLS: MessageListenerService,
    NR: NotifyRepository,
    OS: OrderService,
{
    listener_service: Arc<MLS>,
    notify_repository: Arc<NR>,
    order_service: Arc<OS>,
}

impl<MLS, NR, OS> MessageServiceImpl<MLS, NR, OS>
where
    MLS: MessageListenerService,
    NR: NotifyRepository,
    OS: OrderService,
{
    pub fn new(
        listener_service: Arc<MLS>,
        notify_repository: Arc<NR>,
        order_service: Arc<OS>,
    ) -> Self {
        Self {
            listener_service,
            notify_repository,
            order_service,
        }
    }
}

#[async_trait]
impl<MLS, NR, OS> MessageService for MessageServiceImpl<MLS, NR, OS>
where
    MLS: MessageListenerService,
    NR: NotifyRepository,
    OS: OrderService,
{
    #[instrument(skip(self, notify))]
    async fn convert_notify_to_dto(
        &self,
        notify: Box<dyn Notify>,
    ) -> Result<NotifyDTO, MessageServiceError> {
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
                    MessageServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                        anyhow!("order service error: {}", e),
                    ))
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

    #[instrument(skip(self, notify))]
    async fn send_to_user(
        &self,
        user_id: UserId,
        mut notify: Box<dyn Notify>,
    ) -> Result<(), MessageServiceError> {
        let notify_dto = self
            .convert_notify_to_dto(notify.clone())
            .await
            .inspect_err(|e| {
                error!("Failed to convert notify to DTO: {:?}", e);
            })?;

        let message = Message::from(notify_dto);

        let message_bytes = serde_json::to_vec(&message).unwrap();

        let listener_list = self.listener_service.find_listener_by_user_id(user_id);

        for mut listener in listener_list {
            let _ = listener.on_message(message_bytes.clone()).await;
        }

        self.notify_repository
            .save(notify.as_mut())
            .await
            .inspect_err(|e| {
                error!("Failed to save notify: {:?}", e);
            })
            .map_err(|e| {
                MessageServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Box<dyn Notify>>, MessageServiceError> {
        self.notify_repository
            .load_by_user_id(user_id)
            .await
            .inspect_err(|e| {
                error!("Failed to get message history: {:?}", e);
            })
            .map_err(|e| MessageServiceError::InfrastructureError(ServiceError::RepositoryError(e)))
    }
}
