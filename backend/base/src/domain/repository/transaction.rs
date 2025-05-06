use crate::domain::model::transaction::Transaction;
use crate::domain::model::user::UserId;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait TransactionRepository: Repository<Transaction> {
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Transaction>, RepositoryError>;

    async fn find_by_user_id(&self, user_id: UserId) -> Result<Vec<Transaction>, RepositoryError>;

    async fn get_user_balance(&self, user_id: UserId) -> Result<Option<Decimal>, RepositoryError>;
}
