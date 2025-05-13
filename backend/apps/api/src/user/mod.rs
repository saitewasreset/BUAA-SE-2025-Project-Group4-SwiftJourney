use actix_web::web;

pub mod user_info;
pub mod user_manager;
pub mod personal_info;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(user_info::get_user_info);
    cfg.service(user_info::set_user_info);
    cfg.service(user_manager::register);
    cfg.service(user_manager::login);
    cfg.service(user_manager::logout);
    cfg.service(user_manager::update_password);
    cfg.service(personal_info::get_personal_info);
    cfg.service(personal_info::set_personal_info);
}