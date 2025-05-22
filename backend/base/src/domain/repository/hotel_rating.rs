use crate::domain::model::hotel::{HotelId, HotelRating, Rating};
use crate::domain::model::user::UserId;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

#[async_trait]
pub trait HotelRatingRepository: Repository<HotelRating> {
    async fn get_comments_by_hotel_id(
        &self,
        hotel_id: HotelId,
    ) -> Result<Vec<HotelRating>, RepositoryError>;

    async fn get_comments_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<HotelRating>, RepositoryError>;

    async fn get_hotel_rating(&self, hotel_id: HotelId) -> Result<Option<Rating>, RepositoryError>;
}
