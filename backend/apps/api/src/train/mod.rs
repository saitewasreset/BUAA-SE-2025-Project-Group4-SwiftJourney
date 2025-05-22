use actix_web::web;
use order::new;
use schedule::query;

pub mod order;
pub mod schedule;

// Step 5: Register your endpoint
// HINT: You may refer to `api/user/mod.rs` for example
// Exercise 1.2.1D - 7: Your code here. (4 / 5)
// To `api/main.rs` for following exercise
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(query::query_direct);
    // cfg.service(query::query_indirect);

    cfg.service(new::create_train_order);
}
