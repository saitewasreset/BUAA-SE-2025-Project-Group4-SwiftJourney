use async_trait::async_trait;
use shared::domain::ServiceError;
use shared::domain::model::order::{Order, OrderStatus};
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
    async fn notify_status_change(
        &self,
        transaction_uuid: Uuid,
        atomic: bool,
        orders: &[&dyn Order],
        new_status: OrderStatus,
    );

    async fn order_status_daemon(&self);
}
