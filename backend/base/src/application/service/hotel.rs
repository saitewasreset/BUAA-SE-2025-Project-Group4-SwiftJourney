use crate::application::ApplicationError;
use crate::application::commands::hotel::{NewCommentCommand, QuotaQuery};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HotelServiceError {
    #[error("invalid begin/end date: {0} - {1}")]
    InvalidDateRange(NaiveDate, NaiveDate),
    #[error("invalid rating: {0}")]
    InvalidRating(f64),
    #[error("comment length exceed: {actual} < {limit}")]
    CommentLengthExceed { limit: usize, actual: usize },
    #[error("comment count exceed")]
    CommentCountExceed,
}

impl ApplicationError for HotelServiceError {
    fn error_code(&self) -> u32 {
        match self {
            HotelServiceError::InvalidDateRange(_, _) => 21001,
            HotelServiceError::InvalidRating(_) => 21002,
            HotelServiceError::CommentLengthExceed { .. } => 21003,
            HotelServiceError::CommentCountExceed => 21004,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct HotelCommentQuotaDTO {
    pub quota: i32,
    pub used: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewHotelCommentDTO {
    pub hotel_id: Uuid,
    pub rating: f64,
    pub comment: String,
}

#[async_trait]
pub trait HotelService: 'static + Send + Sync {
    async fn get_quota(
        &self,
        query: QuotaQuery,
    ) -> Result<HotelCommentQuotaDTO, Box<dyn ApplicationError>>;

    async fn new_comment(
        &self,
        command: NewCommentCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;
}
