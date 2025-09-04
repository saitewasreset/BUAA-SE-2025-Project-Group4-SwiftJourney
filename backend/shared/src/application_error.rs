use crate::domain::ServiceError;
use crate::{
    API_BAD_REQUEST_MESSAGE_TEMPLATE, API_FORBIDDEN_MESSAGE_TEMPLATE,
    API_INTERNAL_SERVER_ERROR_MESSAGE, API_NOT_FOUND_MESSAGE_TEMPLATE,
};
use chrono::NaiveDate;
use dyn_fmt::AsStrFormatExt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

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
    #[error("{0}")]
    NotFound(String),
    /// 服务器内部错误
    #[error("an internal server error occurred")]
    InternalServerError,
}

impl ApplicationError for GeneralError {
    fn error_code(&self) -> u32 {
        match self {
            GeneralError::BadRequest(_) => 400,
            GeneralError::InvalidSessionId => 403,
            GeneralError::NotFound(_) => 404,
            GeneralError::InternalServerError => 500,
        }
    }

    fn error_message(&self) -> String {
        match self {
            GeneralError::BadRequest(info) => API_BAD_REQUEST_MESSAGE_TEMPLATE.format(&[info]),
            GeneralError::InvalidSessionId => {
                API_FORBIDDEN_MESSAGE_TEMPLATE.format(&["invalid session id"])
            }
            GeneralError::NotFound(info) => API_NOT_FOUND_MESSAGE_TEMPLATE.format(&[info]),
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

#[derive(Debug, Error)]
pub enum HotelServiceError {
    #[error("invalid begin/end date: {0} - {1}")]
    InvalidDateRange(NaiveDate, NaiveDate),
    // 范围可能无效，所以或许使用字符串传递一个参数更好？
    #[error("invalid date range: {0}")]
    InvalidDateRangeMessage(String),
    #[error("invalid rating: {0}")]
    InvalidRating(f64),
    #[error("comment length exceed: {actual} < {limit}")]
    CommentLengthExceed { limit: usize, actual: usize },
    #[error("comment count exceed")]
    CommentCountExceed,
    #[error("target not found: {0}")]
    TargetNotFound(String),
}

impl ApplicationError for HotelServiceError {
    fn error_code(&self) -> u32 {
        match self {
            HotelServiceError::InvalidDateRange(_, _) => 21001,
            HotelServiceError::InvalidDateRangeMessage(_) => 21001,
            HotelServiceError::InvalidRating(_) => 21002,
            HotelServiceError::CommentLengthExceed { .. } => 21003,
            HotelServiceError::CommentCountExceed => 21004,
            HotelServiceError::TargetNotFound(_) => 404,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug)]
pub enum PersonalInfoError {
    #[error("Identity card id format")]
    InvalidIdentityCardIdFormat,

    #[error("Invalid identity card id")]
    InvalidIdentityCardId,

    #[error("Invalid preferred seat location")]
    InvalidPreferredSeatLocation,
}

impl ApplicationError for PersonalInfoError {
    fn error_code(&self) -> u32 {
        match self {
            Self::InvalidIdentityCardIdFormat => 13001,
            Self::InvalidIdentityCardId => 13002,
            Self::InvalidPreferredSeatLocation => 13003,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Error)]
pub enum TrainDishApplicationServiceError {
    #[error("Invalid dish name: {0}")]
    InvalidDishName(String),
    #[error("Invalid dish name")]
    InvalidAmount,
    #[error("Invalid takeaway station: {0}")]
    InvalidTakeawayStation(String),
    #[error("Invalid takeaway shop name: {0}")]
    InvalidTakeawayShopName(String),
    #[error("Invalid takeaway name: {0}")]
    InvalidTakeawayName(String),
    #[error("No related train order found")]
    NoRelatedTrainOrder,
}

impl ApplicationError for TrainDishApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainDishApplicationServiceError::InvalidDishName(_) => 22001,
            TrainDishApplicationServiceError::InvalidAmount => 22002,
            TrainDishApplicationServiceError::InvalidTakeawayStation(_) => 22003,
            TrainDishApplicationServiceError::InvalidTakeawayShopName(_) => 22004,
            TrainDishApplicationServiceError::InvalidTakeawayName(_) => 22005,
            TrainDishApplicationServiceError::NoRelatedTrainOrder => 22006,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug)]
pub enum TrainOrderServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    /// 会话无效
    #[error("invalid session id:")]
    InvalidSessionId,
    /// 车次号不存在
    #[error("invalid train number")]
    InvalidTrainNumber,
    /// 始发站或终到站不存在
    #[error("invalid station id")]
    InvalidStationId,
    /// 乘车人 Id 不存在，或未与当前用户绑定
    #[error("invalid passenger id")]
    InvalidPassengerId,
}

impl ApplicationError for TrainOrderServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainOrderServiceError::InfrastructureError(_) => 500,
            TrainOrderServiceError::InvalidSessionId => 403,
            TrainOrderServiceError::InvalidTrainNumber => 404,
            TrainOrderServiceError::InvalidStationId => 404,
            TrainOrderServiceError::InvalidPassengerId => 404,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug)]
pub enum TrainQueryServiceError {
    /// 会话无效
    #[error("invalid session id:")]
    InvalidSessionId,
    /// 始发站或终到站不存在
    #[error("invalid station id")]
    InvalidStationId,
    /// 始发城市或终到城市不存在
    #[error("invalid city id")]
    InvalidCityId,
    /// 不满足查询一致性要求
    #[error("inconsistent query")]
    InconsistentQuery,
}

impl ApplicationError for TrainQueryServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TrainQueryServiceError::InvalidSessionId => 403,
            TrainQueryServiceError::InvalidStationId => 404,
            TrainQueryServiceError::InvalidCityId => 404,
            TrainQueryServiceError::InconsistentQuery => 12001,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug)]
pub enum TransactionApplicationServiceError {
    #[error("wrong payment password")]
    WrongPaymentPassword,
    #[error("wrong user password")]
    WrongUserPassword,
    #[error("too many payment password attempts")]
    TooManyPaymentPasswordAttempts,
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("cannot refund this transaction: {0}")]
    RefundError(String),
    #[error("{0}")]
    InvalidTransactionStatus(String),
    #[error("invalid payment password format")]
    InvalidPaymentPasswordFormat,
}

impl ApplicationError for TransactionApplicationServiceError {
    fn error_code(&self) -> u32 {
        match self {
            TransactionApplicationServiceError::WrongPaymentPassword => 11001,
            TransactionApplicationServiceError::WrongUserPassword => 11002,
            TransactionApplicationServiceError::TooManyPaymentPasswordAttempts => 11003,
            TransactionApplicationServiceError::InsufficientFunds => 11004,
            TransactionApplicationServiceError::RefundError(_) => 11005,
            TransactionApplicationServiceError::InvalidTransactionStatus(_) => 11006,
            TransactionApplicationServiceError::InvalidPaymentPasswordFormat => 11007,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

/// 用户管理服务错误类型
///
/// 定义了用户管理服务可能返回的所有特定错误。
#[derive(Error, Debug)]
pub enum UserManagerError {
    /// 用户已存在错误(手机号已注册)
    #[error("User with phone {0} already exists")]
    PhoneAlreadyExists(String),
    /// 用户已存在错误(手机号已注册)
    #[error("User with identity card id {0} already exists")]
    IdentityCardIdAlreadyExists(String),
    /// 手机号或密码错误
    #[error("Invalid phone number or password")]
    InvalidPhoneNumberOrPassword,
    #[error("Invalid username")]
    /// 无效的用户名格式，详见RFC3
    InvalidUsernameFormat,
    #[error("Invalid password")]
    /// 无效的密码格式，详见RFC3
    InvalidPasswordFormat,
    #[error("Invalid name")]
    /// 无效的姓名格式，详见RFC3
    InvalidNameFormat,
}

impl ApplicationError for UserManagerError {
    fn error_code(&self) -> u32 {
        match self {
            UserManagerError::PhoneAlreadyExists(_) => 15001,
            UserManagerError::IdentityCardIdAlreadyExists(_) => 15001,
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

#[derive(Error, Debug)]
pub enum UserProfileError {
    #[error("Invalid age")]
    InvalidAge,
    #[error("Invalid email")]
    InvalidEmail,
    #[error("Invalid username")]
    InvalidUsername,
}

impl ApplicationError for UserProfileError {
    fn error_code(&self) -> u32 {
        match self {
            UserProfileError::InvalidUsername => 15003,
            UserProfileError::InvalidAge => 15006,
            UserProfileError::InvalidEmail => 15007,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
