use actix_web::web;

pub mod order;
pub mod schedule;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/schedule").configure(schedule::scoped_config));
    cfg.service(web::scope("/order").configure(order::scoped_config));
}
