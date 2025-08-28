#![cfg(test)]

use async_trait::async_trait;

use mockall::mock;

use uuid::Uuid;
use crate::domain::model::hotel::{HotelRating, Rating};
use crate::domain::model::user::UserId;
use crate::domain::service::hotel_rating::{HotelRatingService, HotelRatingServiceError};

mock! {
    pub HotelRatingService {}
    
    #[async_trait]
    impl HotelRatingService for HotelRatingService {
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
}

pub fn mock_hotel_rating_service() -> impl HotelRatingService {
    MockHotelRatingService::new()
}