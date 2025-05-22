use actix_web::web;

mod geo;
pub mod mode;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(mode::get_mode)
        .service(geo::get_city_station_info)
        .service(geo::get_city_info);
}
