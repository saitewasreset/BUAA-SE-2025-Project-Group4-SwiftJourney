use crate::domain::model::transaction::Transaction;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TransactionRepository: Repository<Transaction> {
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Transaction>, RepositoryError>;
}
