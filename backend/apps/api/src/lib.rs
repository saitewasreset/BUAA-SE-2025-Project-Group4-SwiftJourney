pub mod train;
pub mod user;

pub mod general;

pub mod data;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::web::Bytes;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use base::application::ApplicationError;
use dyn_fmt::AsStrFormatExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shared::{
    API_BAD_REQUEST_MESSAGE_TEMPLATE, API_FORBIDDEN_CODE, API_FORBIDDEN_MESSAGE_TEMPLATE,
    API_SUCCESS_CODE, API_SUCCESS_MESSAGE,
};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

pub const MAX_BODY_LENGTH: usize = 64 * 1024 * 1024;

pub struct ApplicationErrorBox(pub Box<dyn ApplicationError>);

impl Display for ApplicationErrorBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Debug for ApplicationErrorBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl From<Box<dyn ApplicationError>> for ApplicationErrorBox {
    fn from(value: Box<dyn ApplicationError>) -> Self {
        Self(value)
    }
}

impl From<ApplicationErrorBox> for Box<dyn ApplicationError> {
    fn from(value: ApplicationErrorBox) -> Self {
        value.0
    }
}

impl ResponseError for ApplicationErrorBox {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let api_response: ApiResponse<()> = ApiResponse {
            code: self.0.error_code(),
            message: self.0.error_message(),
            data: None,
        };

        let body = serde_json::to_string(&api_response).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[derive(Debug)]
pub struct ModeError;

impl Display for ModeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModeError")
    }
}

impl std::error::Error for ModeError {}

impl ApplicationError for ModeError {
    fn error_code(&self) -> u32 {
        API_FORBIDDEN_CODE
    }

    fn error_message(&self) -> String {
        API_FORBIDDEN_MESSAGE_TEMPLATE.format(["debug mode is not enabled"])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppConfig {
    pub debug: bool,
}

#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Result<Self, ApplicationErrorBox> {
        Ok(ApiResponse {
            code: API_SUCCESS_CODE,
            message: API_SUCCESS_MESSAGE.to_string(),
            data: Some(data),
        })
    }
}

impl<T> Responder for ApiResponse<T>
where
    T: Serialize,
{
    type Body = BoxBody;
    fn respond_to(self, _for_super_earth: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl<T> From<Result<T, Box<dyn ApplicationError>>> for ApiResponse<T>
where
    T: Serialize,
{
    fn from(value: Result<T, Box<dyn ApplicationError>>) -> Self {
        match value {
            Ok(data) => ApiResponse {
                code: API_SUCCESS_CODE,
                message: API_SUCCESS_MESSAGE.to_string(),
                data: Some(data),
            },
            Err(err) => ApiResponse {
                code: err.error_code(),
                message: err.error_message(),
                data: None,
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum SessionIdError {
    #[error("no session id provided")]
    NoSessionIdProvided,
}

impl ApplicationError for SessionIdError {
    fn error_code(&self) -> u32 {
        403
    }

    fn error_message(&self) -> String {
        API_FORBIDDEN_MESSAGE_TEMPLATE.format(&[self.to_string()])
    }
}

pub fn get_session_id(request: &HttpRequest) -> Result<String, Box<dyn ApplicationError>> {
    if let Some(provided_session_id) = request.cookie("session_id") {
        Ok(provided_session_id.value().to_string())
    } else {
        Err(SessionIdError::NoSessionIdProvided.into())
    }
}

#[derive(Error, Debug)]
pub enum ParseRequestBodyError {
    #[error("invalid request body")]
    InvalidBody(#[from] serde_json::Error),
}

impl ApplicationError for ParseRequestBodyError {
    fn error_code(&self) -> u32 {
        400
    }

    fn error_message(&self) -> String {
        API_BAD_REQUEST_MESSAGE_TEMPLATE.format(&["invalid json"])
    }
}

pub fn parse_request_body<T: DeserializeOwned>(
    raw_body: Bytes,
) -> Result<T, Box<dyn ApplicationError>> {
    serde_json::from_slice(&raw_body)
        .map_err(|e| Box::new(ParseRequestBodyError::InvalidBody(e)) as Box<dyn ApplicationError>)
}
