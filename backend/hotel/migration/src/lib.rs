pub use sea_orm_migration::prelude::*;

mod m20250411_010603_create_city;
mod m20250411_010614_create_station;
mod m20250411_010715_create_user;
mod m20250411_010719_create_person_info;
mod m20250411_010744_create_hotel;
mod m20250411_010751_create_hotel_room_type;
mod m20250411_010807_create_hotel_rating;
mod m20250411_010814_create_occupied_room;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250411_010603_create_city::Migration),
            Box::new(m20250411_010614_create_station::Migration),
            Box::new(m20250411_010715_create_user::Migration),
            Box::new(m20250411_010719_create_person_info::Migration),
            Box::new(m20250411_010744_create_hotel::Migration),
            Box::new(m20250411_010751_create_hotel_room_type::Migration),
            Box::new(m20250411_010807_create_hotel_rating::Migration),
            Box::new(m20250411_010814_create_occupied_room::Migration),
        ]
    }
}
