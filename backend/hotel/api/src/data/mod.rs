use actix_web::web;

pub mod hotel;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(hotel::load_hotel_data);
}
