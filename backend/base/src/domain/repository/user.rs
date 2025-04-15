use crate::domain::Repository;
use crate::domain::model::user::User;

pub trait UserRepository: Repository<User> {}
