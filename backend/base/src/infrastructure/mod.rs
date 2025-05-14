pub mod application;
#[macro_use]
pub mod macros;
pub mod messaging;
pub mod repository;
pub mod service;

pub const RABBITMQ_ORDER_STATUS_EXCHANGE_NAME: &str = "order_status_exchange";
