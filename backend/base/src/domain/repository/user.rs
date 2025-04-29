use crate::domain::model::user::{IdentityCardId, Phone, User};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Repository<User> + 'static + Send + Sync {
    async fn find_by_phone(&self, phone: Phone) -> Result<Option<User>, RepositoryError>;

    async fn find_by_identity_card_id(
        &self,
        identity_card_id: IdentityCardId,
    ) -> Result<Option<User>, RepositoryError>;

    async fn remove_by_phone(&self, phone: Phone) -> Result<(), RepositoryError>;
}
