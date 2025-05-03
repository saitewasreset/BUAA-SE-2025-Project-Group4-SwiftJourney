use actix_web::web;

pub mod mode;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(mode::get_mode);
}
