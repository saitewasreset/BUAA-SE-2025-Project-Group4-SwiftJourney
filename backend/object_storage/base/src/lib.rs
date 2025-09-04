/*
* Even if we lose our way, we keep on moving.
*/

pub mod application;
pub mod domain;
pub mod infrastructure;

pub trait DbId {
    type DbType;

    fn to_db_value(&self) -> Self::DbType;
    fn from_db_value(value: Self::DbType) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}
