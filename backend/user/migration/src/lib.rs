pub use sea_orm_migration::prelude::*;
mod m20250411_010715_create_user;
mod m20250411_010719_create_person_info;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250411_010715_create_user::Migration),
            Box::new(m20250411_010719_create_person_info::Migration),
        ]
    }
}
