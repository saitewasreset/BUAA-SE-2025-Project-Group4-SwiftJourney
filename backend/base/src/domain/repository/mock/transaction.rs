#![cfg(test)]

use crate::domain::model::transaction::{Transaction, TransactionId};
use crate::domain::model::user::UserId;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use mockall::mock;
use rust_decimal::Decimal;
use uuid::Uuid;
use crate::domain::repository::transaction::TransactionRepository;

mock! {
    pub TransactionRepository {}

    #[async_trait]
    impl TransactionRepository for TransactionRepository {
        async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Transaction>, RepositoryError>;

        async fn find_by_user_id(&self, user_id: UserId) -> Result<Vec<Transaction>, RepositoryError>;

        async fn get_user_balance(&self, user_id: UserId) -> Result<Option<Decimal>, RepositoryError>;
    }

    #[async_trait]
    impl Repository<Transaction> for TransactionRepository {
        async fn find(&self, id: TransactionId) -> Result<Option<Transaction>, RepositoryError>;
        async fn remove(&self, aggregate: Transaction) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut Transaction) -> Result<TransactionId, RepositoryError>;
    }
}

pub fn mock_transaction_repository() -> MockTransactionRepository {
    MockTransactionRepository::new()
}