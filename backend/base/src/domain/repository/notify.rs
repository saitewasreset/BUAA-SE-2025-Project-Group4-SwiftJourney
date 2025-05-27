use crate::domain::RepositoryError;
use crate::domain::model::message::{Notify, NotifyId, OrderNotify, TripNotify};
use crate::domain::model::user::UserId;
use async_trait::async_trait;

#[async_trait]
pub trait NotifyRepository: 'static + Send + Sync {
    async fn find(&self, notify_id: NotifyId) -> Result<Option<Box<dyn Notify>>, RepositoryError>;

    async fn find_order(&self, notify_id: NotifyId)
    -> Result<Option<OrderNotify>, RepositoryError>;

    async fn find_trip(&self, notify_id: NotifyId) -> Result<Option<TripNotify>, RepositoryError>;

    async fn remove(&self, notify_id: NotifyId) -> Result<(), RepositoryError>;

    async fn save(&self, notify: &mut dyn Notify) -> Result<NotifyId, RepositoryError>;

    async fn load_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Box<dyn Notify>>, RepositoryError>;
}
