use actix_web::web;

pub mod query;

use crate::train::schedule::query::{query_direct, query_indirect};

// Step 4: Register your endpoint
// HINT: You may refer to `api/user/mod.rs` for example
// Exercise 1.2.1D - 7: Your code here. (3 / 5)
// To `api/train/mod.rs` for following exercise
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_direct);
    cfg.service(query_indirect);
}
