use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum OrderStatusManagerServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("order {0} status is invalid: {1}")]
    InvalidStatus(Uuid, OrderStatus),
}
#[async_trait]
pub trait OrderStatusManagerService: 'static + Send + Sync {
    async fn attach(&self, order: &dyn Order) -> Result<(), OrderStatusManagerServiceError>;
    async fn detach(&self, order: &dyn Order) -> Result<(), OrderStatusManagerServiceError>;

    async fn notify_status_change(&self, order_uuid: Uuid, new_status: OrderStatus);
}
