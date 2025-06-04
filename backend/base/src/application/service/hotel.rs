use std::collections::HashMap;

use crate::application::ApplicationError;
use crate::application::commands::hotel::{
    HotelInfoQuery, HotelOrderInfoQuery, HotelQuery, NewCommentCommand, QuotaQuery,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelGeneralInfoDTO {
    pub hotel_id: Uuid,
    pub name: String,
    pub picture: Option<String>,
    pub rating: f64,
    pub rating_count: i32,
    pub total_bookings: i32,
    pub price: f64,
    pub info: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelCommentDTO {
    pub user_name: String,
    pub comment_time: String,
    pub rating: f64,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelDetailInfoDTO {
    pub hotel_id: String,
    pub name: String,
    pub address: String,
    pub phone: Vec<String>,
    pub info: String,

    pub picture: Option<Vec<String>>,
    pub rating: f64,
    pub rating_count: i32,
    pub total_bookings: i32,
    pub comments: Vec<HotelCommentDTO>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotelRoomDetailInfoDTO {
    pub capacity: i32,
    pub remain_count: i32,
    pub price: f64,
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

    async fn query_hotels(
        &self,
        query: HotelQuery,
    ) -> Result<Vec<HotelGeneralInfoDTO>, Box<dyn ApplicationError>>;

    async fn query_hotel_info(
        &self,
        query: HotelInfoQuery,
    ) -> Result<HotelDetailInfoDTO, Box<dyn ApplicationError>>;

    async fn query_hotel_order_info(
        &self,
        query: HotelOrderInfoQuery,
    ) -> Result<HashMap<String, HotelRoomDetailInfoDTO>, Box<dyn ApplicationError>>;
}
