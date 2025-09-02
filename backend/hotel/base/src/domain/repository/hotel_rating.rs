use crate::domain::model::hotel::{HotelId, HotelRating, Rating};
use async_trait::async_trait;
use shared::domain::model::user::UserId;
use shared::domain::{Repository, RepositoryError};

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
