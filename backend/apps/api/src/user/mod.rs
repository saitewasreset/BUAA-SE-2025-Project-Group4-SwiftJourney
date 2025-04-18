use actix_web::web;

pub mod user_info;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(user_info::get_user_info);
    cfg.service(user_info::set_user_info);
}
