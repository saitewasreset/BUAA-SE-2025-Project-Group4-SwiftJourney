#![cfg(test)]

use crate::domain::model::order::Order;
use crate::domain::model::transaction::{Transaction, TransactionAmountAbs};
use crate::domain::model::user::UserId;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::transaction::{TransactionService, TransactionServiceError};
use async_trait::async_trait;
use mockall::mock;
use rust_decimal::Decimal;
use uuid::Uuid;

mock! {
    pub TransactionService {}

    #[async_trait]
    impl TransactionService for TransactionService {
        async fn recharge(
        &self,
        user_id: UserId,
        amount: TransactionAmountAbs,
    ) -> Result<Uuid, TransactionServiceError>;

    async fn get_balance(&self, user_id: UserId) -> Result<Decimal, TransactionServiceError>;

    async fn new_transaction(
        &self,
        user_id: UserId,
        orders: Vec<Box<dyn Order>>,
        atomic: bool,
    ) -> Result<Uuid, TransactionServiceError>;

    async fn pay_transaction(&self, transaction_id: Uuid) -> Result<(), TransactionServiceError>;

    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
        to_refund_orders: &[Box<dyn Order>],
    ) -> Result<Uuid, TransactionServiceError>;

    async fn convert_transaction_to_dto(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionDataDto, TransactionServiceError>;
    }
}

pub fn mock_transaction_service() -> MockTransactionService {
    MockTransactionService::new()
}
