#![cfg(test)]

use crate::domain::model::hotel::{HotelId, HotelRating, HotelRatingId, Rating};
use crate::domain::model::user::UserId;
use crate::domain::repository::hotel_rating::HotelRatingRepository;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use mockall::mock;

mock! {
    pub HotelRatingRepository {}

    #[async_trait]
    impl Repository<HotelRating> for HotelRatingRepository {
        async fn find(&self, id: HotelRatingId) -> Result<Option<HotelRating>, RepositoryError>;
        async fn remove(&self, aggregate: HotelRating) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut HotelRating) -> Result<HotelRatingId, RepositoryError>;
    }

    #[async_trait]
    impl HotelRatingRepository for HotelRatingRepository {
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

}

pub fn mock_hotel_rating_repository() -> MockHotelRatingRepository {
    MockHotelRatingRepository::new()
}
