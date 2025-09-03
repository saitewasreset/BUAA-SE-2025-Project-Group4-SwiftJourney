pub mod new;

use actix_web::web;

use crate::train::order::new::create_train_order;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_train_order);
}
