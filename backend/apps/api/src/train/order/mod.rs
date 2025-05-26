pub mod new;

use actix_web::web;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(new::create_train_order);
}
