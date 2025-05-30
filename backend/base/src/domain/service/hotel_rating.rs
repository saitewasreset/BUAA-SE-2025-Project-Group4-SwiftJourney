use crate::domain::RepositoryError;
use crate::domain::model::hotel::{HotelRating, Rating};
use crate::domain::model::user::UserId;
use crate::domain::service::ServiceError;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HotelRatingServiceError {
    /// 底层基础设施错误（如数据库访问失败）
    #[error("an infrastructure error occurred: {0}")]
    InfrastructureError(ServiceError),
    #[error("invalid hotel uuid: {0}")]
    InvalidHotelUuid(Uuid),
    #[error("no comments quota left for hotel uuid: {0}, quota: {1}")]
    NoCommentsQuotaLeft(Uuid, i32),
    #[error("comment length exceed: {actual} < {limit}")]
    CommentLengthExceed { limit: usize, actual: usize },
}

impl From<RepositoryError> for HotelRatingServiceError {
    fn from(value: RepositoryError) -> Self {
        HotelRatingServiceError::InfrastructureError(ServiceError::RepositoryError(value))
    }
}

#[async_trait]
pub trait HotelRatingService: 'static + Send + Sync {
    async fn get_hotel_rating(&self, hotel_uuid: Uuid) -> Result<Rating, HotelRatingServiceError>;

    async fn get_hotel_comment_quota(
        &self,
        hotel_uuid: Uuid,
        user_id: UserId,
    ) -> Result<i32, HotelRatingServiceError>;

    async fn get_current_comment_count(
        &self,
        hotel_uuid: Uuid,
        user_id: UserId,
    ) -> Result<i32, HotelRatingServiceError>;

    async fn get_comments(
        &self,
        hotel_uuid: Uuid,
    ) -> Result<Vec<HotelRating>, HotelRatingServiceError>;

    async fn add_comment(
        &self,
        hotel_uuid: Uuid,
        user_id: UserId,
        rating: Rating,
        text: String,
    ) -> Result<(), HotelRatingServiceError>;
}
