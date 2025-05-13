use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::service::order_status::{
    OrderStatusManagerService, OrderStatusManagerServiceError,
};
use async_trait::async_trait;
use uuid::Uuid;

pub struct OrderStatusManagerServiceImpl {}

impl OrderStatusManagerServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl OrderStatusManagerService for OrderStatusManagerServiceImpl {
    async fn attach(&self, order: &dyn Order) -> Result<(), OrderStatusManagerServiceError> {
        todo!()
    }

    async fn detach(&self, order: &dyn Order) -> Result<(), OrderStatusManagerServiceError> {
        todo!()
    }

    async fn notify_status_change(&self, order: &dyn Order, new_status: OrderStatus) {
        todo!()
    }
}
