use actix_web::web;

pub mod city_info;
pub mod city_station_info;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(city_info::get_city_info)
        .service(city_station_info::get_city_station_info);
}
