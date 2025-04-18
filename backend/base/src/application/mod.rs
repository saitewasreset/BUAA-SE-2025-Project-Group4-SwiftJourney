pub mod commands;
pub mod service;

pub trait ApplicationError: std::error::Error + 'static {
    fn error_code(&self) -> u32;
    fn error_message(&self) -> String;
}

impl<T> From<T> for Box<dyn ApplicationError>
where
    T: ApplicationError,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}
