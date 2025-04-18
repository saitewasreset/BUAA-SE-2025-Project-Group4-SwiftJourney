pub mod commands;
pub mod service;

pub trait ApplicationError: std::error::Error + 'static {
    fn error_code(&self) -> u32;
    fn error_message(&self) -> String;
}
