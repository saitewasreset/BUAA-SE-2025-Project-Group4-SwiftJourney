#![cfg(test)]

use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::service::order_status::OrderStatusManagerService;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 手动 Mock 的 OrderStatusManagerService
#[derive(Clone, Default)]
pub struct MockOrderStatusManagerService {
    /// 可以记录调用次数或参数，用于测试验证
    pub notified: Arc<Mutex<Vec<(Uuid, bool, Vec<Uuid>, OrderStatus)>>>,
}

#[async_trait::async_trait]
impl OrderStatusManagerService for MockOrderStatusManagerService {
    async fn notify_status_change(
        &self,
        transaction_uuid: Uuid,
        atomic: bool,
        orders: &[&dyn Order],
        new_status: OrderStatus,
    ) {
        let order_uuids = orders.iter().map(|o| o.uuid()).collect();
        self.notified
            .lock()
            .unwrap()
            .push((transaction_uuid, atomic, order_uuids, new_status));
    }

    async fn order_status_daemon(&self) {
        // do nothing in mock
    }
}

/// 构造函数
pub fn mock_order_status_manager_service() -> MockOrderStatusManagerService {
    MockOrderStatusManagerService::default()
}
