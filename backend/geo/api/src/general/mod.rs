use actix_web::web;

mod geo;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(geo::get_city_station_info)
        .service(geo::get_city_info);
}
