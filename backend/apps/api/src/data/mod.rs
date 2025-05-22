use actix_web::web;

pub mod hotel;
pub mod train;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(train::load_city_data)
        .service(train::load_station_data)
        .service(train::load_train_type_data)
        .service(train::load_train_number_data)
        .service(hotel::load_hotel_data);
}
