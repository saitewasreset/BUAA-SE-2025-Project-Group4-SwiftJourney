use crate::application::ApplicationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersonalInfoError {
    ///身份证号格式错误
    #[error("Invalid identity card id format")]
    InvalidIdentityCardIdFormat,
    ///该身份证号对应的个人信息不存在，或没有权限设置
    #[error("invalid identity card id")]
    InvalidIdentityCardId,
}

impl ApplicationError for PersonalInfoError {
    fn error_code(&self) -> u32 {
        match self {
            PersonalInfoError::InvalidIdentityCardIdFormat => 13001,
            PersonalInfoError::InvalidIdentityCardId => 13002,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
