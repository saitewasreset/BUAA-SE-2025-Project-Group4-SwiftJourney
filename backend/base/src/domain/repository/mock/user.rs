#![cfg(test)]

use crate::domain::model::user::{IdentityCardId, Phone, User, UserId};
use crate::domain::repository::user::UserRepository;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use mockall::mock;

mock! {
    pub UserRepository {}

    #[async_trait]
    impl UserRepository for UserRepository {
        async fn find_by_phone(&self, phone: Phone) -> Result<Option<User>, RepositoryError>;

        async fn find_by_identity_card_id(
            &self,
            identity_card_id: IdentityCardId,
        ) -> Result<Option<User>, RepositoryError>;

        async fn remove_by_phone(&self, phone: Phone) -> Result<(), RepositoryError>;
    }

    // Repository<User> 这个 trait 也要 mock
    #[async_trait]
    impl Repository<User> for UserRepository {
        async fn find(&self, id: UserId) -> Result<Option<User>, RepositoryError>;
        async fn remove(&self, aggregate: User) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut User) -> Result<UserId, RepositoryError>;
    }
}

pub fn mock_user() -> MockUserRepository {
    MockUserRepository::new()
}
