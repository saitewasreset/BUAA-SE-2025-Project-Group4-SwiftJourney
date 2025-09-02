pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod models;

pub use shared::{Unverified, Verified};

pub trait DbId {
    type DbType;

    fn to_db_value(&self) -> Self::DbType;
    fn from_db_value(value: Self::DbType) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}
