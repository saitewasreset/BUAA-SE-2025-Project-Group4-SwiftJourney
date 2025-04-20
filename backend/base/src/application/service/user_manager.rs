use crate::application::commands::user_manager::{
    UserLoginCommand, UserLogoutCommand, UserRegisterCommand,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::session::SessionId;
use crate::domain::service::user::UserServiceError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserRegisterDTO {
    pub phone: String,
    pub username: String,
    pub password: String,
    pub name: String,
    #[serde(rename = "identityCardId")]
    pub identity_card_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLoginDTO {
    pub phone: String,
    pub password: String,
}

#[derive(Error, Debug)]
pub enum UserManagerError {
    #[error("User with phone {0} already exists")]
    UserAlreadyExists(String),
    #[error("Invalid phone number of password")]
    InvalidPhoneNumberOrPassword,
    #[error("Invalid username")]
    InvalidUsernameFormat,
    #[error("Invalid password")]
    InvalidPasswordFormat,
    #[error("Invalid name")]
    InvalidNameFormat,
}

impl ApplicationError for UserManagerError {
    fn error_code(&self) -> u32 {
        match self {
            UserManagerError::UserAlreadyExists(_) => 15001,
            UserManagerError::InvalidPhoneNumberOrPassword => 15002,
            UserManagerError::InvalidUsernameFormat => 15003,
            UserManagerError::InvalidPasswordFormat => 15004,
            UserManagerError::InvalidNameFormat => 15005,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

impl From<UserServiceError> for Box<dyn ApplicationError> {
    fn from(value: UserServiceError) -> Self {
        match value {
            UserServiceError::InfrastructureError(_) => GeneralError::InternalServerError.into(),
            UserServiceError::InvalidPassword => {
                UserManagerError::InvalidPhoneNumberOrPassword.into()
            }
            UserServiceError::NoSuchUser(_) => {
                UserManagerError::InvalidPhoneNumberOrPassword.into()
            }
            UserServiceError::UserExists(s, _) => UserManagerError::UserAlreadyExists(s).into(),
            UserServiceError::PaymentPasswordMaxAttemptsExceed(_) => todo!("应在交易模块实现"),
        }
    }
}

#[async_trait]
pub trait UserManagerService {
    async fn register(&self, command: UserRegisterCommand)
    -> Result<(), Box<dyn ApplicationError>>;
    async fn login(
        &self,
        command: UserLoginCommand,
    ) -> Result<SessionId, Box<dyn ApplicationError>>;
    async fn logout(&self, command: UserLogoutCommand) -> Result<(), Box<dyn ApplicationError>>;
}
