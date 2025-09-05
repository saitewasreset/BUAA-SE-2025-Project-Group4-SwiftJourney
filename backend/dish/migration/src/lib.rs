pub use sea_orm_migration::prelude::*;

mod m20250411_010614_create_station;
mod m20250411_010617_create_train;
mod m20250411_010701_create_route;
mod m20250411_010818_create_dish;
mod m20250411_010825_create_takeaway_shop;
mod m20250411_010827_create_takeaway_dish;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250411_010614_create_station::Migration),
            Box::new(m20250411_010617_create_train::Migration),
            Box::new(m20250411_010701_create_route::Migration),
            Box::new(m20250411_010818_create_dish::Migration),
            Box::new(m20250411_010825_create_takeaway_shop::Migration),
            Box::new(m20250411_010827_create_takeaway_dish::Migration),
        ]
    }
}
