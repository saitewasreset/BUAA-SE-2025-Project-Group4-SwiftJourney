use actix_web::web;

pub mod order;
pub mod schedule;

// Step 5: Register your endpoint
// HINT: You may refer to `api/user/mod.rs` for example
// Exercise 1.2.1D - 7: Your code here. (4 / 5)
// To `api/main.rs` for following exercise
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/schedule").configure(schedule::scoped_config));
    cfg.service(web::scope("/order").configure(order::scoped_config));
}
