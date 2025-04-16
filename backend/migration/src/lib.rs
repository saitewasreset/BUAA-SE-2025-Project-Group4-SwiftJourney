pub use sea_orm_migration::prelude::*;

mod m20250411_010603_create_city;
mod m20250411_010610_create_train_type;
mod m20250411_010614_create_station;
mod m20250411_010617_create_train;
mod m20250411_010621_create_seat_type;
mod m20250411_010655_create_seat_type_in_train_type;
mod m20250411_010701_create_route;
mod m20250411_010708_create_train_route;
mod m20250411_010715_create_user;
mod m20250411_010719_create_person_info;
mod m20250411_010725_create_transaction;
mod m20250411_010735_create_occupied_seat;
mod m20250411_010744_create_hotel;
mod m20250411_010751_create_hotel_room_type;
mod m20250411_010807_create_hotel_rating;
mod m20250411_010814_create_occupied_room;
mod m20250411_010818_create_dish;
mod m20250411_010825_create_takeaway_shop;
mod m20250411_010827_create_takeaway_dish;
mod m20250411_010837_create_train_order;
mod m20250411_010843_create_hotel_order;
mod m20250411_010855_create_dish_order;
mod m20250411_010858_create_takeaway_order;
mod m20250411_010905_create_message;
mod m20250411_011731_seat_type_mapping;
mod m20250416_064747_modify_user_hashed_payment_password_as_nullable;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250411_010603_create_city::Migration),
            Box::new(m20250411_010610_create_train_type::Migration),
            Box::new(m20250411_010614_create_station::Migration),
            Box::new(m20250411_010617_create_train::Migration),
            Box::new(m20250411_010621_create_seat_type::Migration),
            Box::new(m20250411_010655_create_seat_type_in_train_type::Migration),
            Box::new(m20250411_010701_create_route::Migration),
            Box::new(m20250411_010708_create_train_route::Migration),
            Box::new(m20250411_010715_create_user::Migration),
            Box::new(m20250411_010719_create_person_info::Migration),
            Box::new(m20250411_010725_create_transaction::Migration),
            Box::new(m20250411_010735_create_occupied_seat::Migration),
            Box::new(m20250411_010744_create_hotel::Migration),
            Box::new(m20250411_010751_create_hotel_room_type::Migration),
            Box::new(m20250411_010807_create_hotel_rating::Migration),
            Box::new(m20250411_010814_create_occupied_room::Migration),
            Box::new(m20250411_010818_create_dish::Migration),
            Box::new(m20250411_010825_create_takeaway_shop::Migration),
            Box::new(m20250411_010827_create_takeaway_dish::Migration),
            Box::new(m20250411_010837_create_train_order::Migration),
            Box::new(m20250411_010843_create_hotel_order::Migration),
            Box::new(m20250411_010855_create_dish_order::Migration),
            Box::new(m20250411_010858_create_takeaway_order::Migration),
            Box::new(m20250411_010905_create_message::Migration),
            Box::new(m20250411_011731_seat_type_mapping::Migration),
            Box::new(m20250416_064747_modify_user_hashed_payment_password_as_nullable::Migration),
        ]
    }
}
