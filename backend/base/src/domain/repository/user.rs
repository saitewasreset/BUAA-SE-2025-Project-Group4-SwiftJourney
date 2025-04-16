use crate::domain::model::user::{Phone, User};
use crate::domain::{Repository, RepositoryError};

pub trait UserRepository: Repository<User> {
    fn find_by_phone(
        &self,
        phone: Phone,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;
}
