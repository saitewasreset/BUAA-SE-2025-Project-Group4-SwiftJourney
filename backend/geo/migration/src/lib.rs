pub use sea_orm_migration::prelude::*;

mod m20250411_010603_create_city;
mod m20250411_010614_create_station;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250411_010603_create_city::Migration),
            Box::new(m20250411_010614_create_station::Migration),
        ]
    }
}
