pub use sea_orm_migration::prelude::*;

mod m20250411_010603_create_city;
mod m20250411_010610_create_train_type;
mod m20250411_010614_create_station;
mod m20250411_010617_create_train;
mod m20250411_010620_create_train_schedule;
mod m20250411_010621_create_seat_type;
mod m20250411_010655_create_seat_type_in_train_type;
mod m20250411_010701_create_route;
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
mod m20250421_023554_modify_transaction_time_as_nullable;
mod m20250421_073207_modify_train_order_transaction_as_nullable;
mod m20250421_073213_modify_hotel_order_transaction_as_nullable;
mod m20250421_073220_modify_dish_order_transaction_as_nullable;
mod m20250421_073248_modify_takeaway_order_transaction_as_nullable;
mod m20250421_083028_modify_city_add_province;
mod m20250426_031246_modify_station_change_city_id_type;
mod m20250426_041251_modify_train_change_train_type_id_type;
mod m20250426_060711_modify_seat_type_in_train_type_change_type;
mod m20250430_062929_modify_city_add_unique_name;
mod m20250430_063613_modify_station_add_unique_name_city;
mod m20250430_064614_modify_train_add_unique_number;
mod m20250430_065037_modify_train_type_add_unique_type_name;
mod m20250501_024033_modify_route_change_station_id_type;
mod m20250501_034231_modify_train_add_default_line_id;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250411_010603_create_city::Migration),
            Box::new(m20250411_010610_create_train_type::Migration),
            Box::new(m20250411_010614_create_station::Migration),
            Box::new(m20250411_010617_create_train::Migration),
            Box::new(m20250411_010620_create_train_schedule::Migration),
            Box::new(m20250411_010621_create_seat_type::Migration),
            Box::new(m20250411_010655_create_seat_type_in_train_type::Migration),
            Box::new(m20250411_010701_create_route::Migration),
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
            Box::new(m20250421_023554_modify_transaction_time_as_nullable::Migration),
            Box::new(m20250421_073207_modify_train_order_transaction_as_nullable::Migration),
            Box::new(m20250421_073213_modify_hotel_order_transaction_as_nullable::Migration),
            Box::new(m20250421_073220_modify_dish_order_transaction_as_nullable::Migration),
            Box::new(m20250421_073248_modify_takeaway_order_transaction_as_nullable::Migration),
            Box::new(m20250421_083028_modify_city_add_province::Migration),
            Box::new(m20250426_031246_modify_station_change_city_id_type::Migration),
            Box::new(m20250426_041251_modify_train_change_train_type_id_type::Migration),
            Box::new(m20250426_060711_modify_seat_type_in_train_type_change_type::Migration),
            Box::new(m20250430_062929_modify_city_add_unique_name::Migration),
            Box::new(m20250430_063613_modify_station_add_unique_name_city::Migration),
            Box::new(m20250430_064614_modify_train_add_unique_number::Migration),
            Box::new(m20250430_065037_modify_train_type_add_unique_type_name::Migration),
            Box::new(m20250501_024033_modify_route_change_station_id_type::Migration),
            Box::new(m20250501_034231_modify_train_add_default_line_id::Migration),
        ]
    }
}
