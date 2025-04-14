use crate::domain::Repository;
use crate::domain::model::user::{User, UserId};

pub trait UserRepository: Repository<User, UserId> {}
