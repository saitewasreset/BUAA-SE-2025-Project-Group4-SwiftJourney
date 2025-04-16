use crate::domain::model::user::{IdentityCardId, Phone, User};
use crate::domain::{Repository, RepositoryError};

pub trait UserRepository: Repository<User> {
    fn find_by_phone(
        &self,
        phone: Phone,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn find_by_identity_card_id(
        &self,
        identity_card_id: IdentityCardId,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;
}
