use dyn_fmt::AsStrFormatExt;
use shared::{
    API_BAD_REQUEST_MESSAGE_TEMPLATE, API_FORBIDDEN_MESSAGE_TEMPLATE,
    API_INTERNAL_SERVER_ERROR_MESSAGE,
};
use std::fmt::{Display, Formatter};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum GeneralError {
    /// 会话ID无效
    #[error("invalid session id")]
    InvalidSessionId,
    /// 请求参数无效
    #[error("{0}")]
    BadRequest(String),
    /// 请求资源不存在
    #[error("resource not found")]
    NotFound,
    /// 服务器内部错误
    #[error("an internal server error occurred")]
    InternalServerError,
}

impl ApplicationError for GeneralError {
    fn error_code(&self) -> u32 {
        match self {
            GeneralError::BadRequest(_) => 400,
            GeneralError::InvalidSessionId => 403,
            GeneralError::NotFound => 404,
            GeneralError::InternalServerError => 500,
        }
    }

    fn error_message(&self) -> String {
        match self {
            GeneralError::BadRequest(info) => API_BAD_REQUEST_MESSAGE_TEMPLATE.format(&[info]),
            GeneralError::InvalidSessionId => {
                API_FORBIDDEN_MESSAGE_TEMPLATE.format(&["invalid session id"])
            }
            GeneralError::NotFound => "resource not found".to_owned(),
            GeneralError::InternalServerError => API_INTERNAL_SERVER_ERROR_MESSAGE.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct ModeError;

impl Display for ModeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "mode error")
    }
}

impl std::error::Error for ModeError {}

impl ApplicationError for ModeError {
    fn error_code(&self) -> u32 {
        403
    }

    fn error_message(&self) -> String {
        "debug mode is not enabled".to_string()
    }
}
