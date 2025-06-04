use crate::HOTEL_MAX_COMMENT_LENGTH;
use crate::domain::model::hotel::{HotelRating, Rating};
use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::model::user::UserId;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::hotel_rating::HotelRatingRepository;
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::hotel_rating::{HotelRatingService, HotelRatingServiceError};
use async_trait::async_trait;
use chrono::Local;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct HotelRatingServiceImpl<HR, HRR, OR>
where
    HR: HotelRepository,
    HRR: HotelRatingRepository,
    OR: OrderRepository,
{
    hotel_repository: Arc<HR>,
    hotel_rating_repository: Arc<HRR>,
    order_repository: Arc<OR>,
}

impl<HR, HRR, OR> HotelRatingServiceImpl<HR, HRR, OR>
where
    HR: HotelRepository,
    HRR: HotelRatingRepository,
    OR: OrderRepository,
{
    pub fn new(
        hotel_repository: Arc<HR>,
        hotel_rating_repository: Arc<HRR>,
        order_repository: Arc<OR>,
    ) -> Self {
        Self {
            hotel_repository,
            hotel_rating_repository,
            order_repository,
        }
    }

    fn now() -> DateTimeWithTimeZone {
        let local_now = Local::now();
        let offset = *local_now.offset(); // 获取系统当前时区偏移
        local_now.with_timezone(&offset)
    }
}

#[async_trait]
impl<HR, HRR, OR> HotelRatingService for HotelRatingServiceImpl<HR, HRR, OR>
where
    HR: HotelRepository,
    HRR: HotelRatingRepository,
    OR: OrderRepository,
{
    #[instrument(skip(self))]
    async fn get_hotel_rating(&self, hotel_uuid: Uuid) -> Result<Rating, HotelRatingServiceError> {
        if let Some(hotel_id) = self
            .hotel_repository
            .get_id_by_uuid(hotel_uuid)
            .await
            .inspect_err(|e| error!("Failed to get hotel id by uuid {}: {}", hotel_uuid, e))?
        {
            Ok(self
                .hotel_rating_repository
                .get_hotel_rating(hotel_id)
                .await
                .inspect_err(|e| {
                    error!(
                        "Failed to get hotel rating for hotel id {}: {}",
                        hotel_id, e
                    )
                })?
                .unwrap_or_default())
        } else {
            Err(HotelRatingServiceError::InvalidHotelUuid(hotel_uuid))
        }
    }

    #[instrument(skip(self))]
    async fn get_hotel_comment_quota(
        &self,
        hotel_uuid: Uuid,
        user_id: UserId,
    ) -> Result<i32, HotelRatingServiceError> {
        if let Some(hotel_id) = self
            .hotel_repository
            .get_id_by_uuid(hotel_uuid)
            .await
            .inspect_err(|e| error!("Failed to get hotel id by uuid {}: {}", hotel_uuid, e))?
        {
            let current_user_hotel_orders = self
                .order_repository
                .find_hotel_order_by_userid(user_id, hotel_id)
                .await
                .inspect_err(|e| {
                    error!(
                        "Failed to find hotel order by user id {} hotel id {}: {}",
                        user_id, hotel_id, e
                    )
                })?;

            let valid_count: i32 = current_user_hotel_orders
                .iter()
                .filter(|x| x.order_status() == OrderStatus::Completed)
                .map(|x| x.amount().to_i32().unwrap())
                .sum();

            Ok(valid_count)
        } else {
            Err(HotelRatingServiceError::InvalidHotelUuid(hotel_uuid))
        }
    }

    #[instrument(skip(self))]
    async fn get_current_comment_count(
        &self,
        hotel_uuid: Uuid,
        user_id: UserId,
    ) -> Result<i32, HotelRatingServiceError> {
        if let Some(hotel_id) = self
            .hotel_repository
            .get_id_by_uuid(hotel_uuid)
            .await
            .inspect_err(|e| error!("Failed to get hotel id by uuid {}: {}", hotel_uuid, e))?
        {
            let user_comments = self
                .hotel_rating_repository
                .get_comments_by_user_id(user_id)
                .await
                .inspect_err(|e| error!("Failed to get comments by user id {}: {}", user_id, e))?;

            let count = user_comments
                .iter()
                .filter(|x| x.hotel_id() == hotel_id)
                .count();

            Ok(count as i32)
        } else {
            Err(HotelRatingServiceError::InvalidHotelUuid(hotel_uuid))
        }
    }

    #[instrument(skip(self))]
    async fn get_comments(
        &self,
        hotel_uuid: Uuid,
    ) -> Result<Vec<HotelRating>, HotelRatingServiceError> {
        if let Some(hotel_id) = self
            .hotel_repository
            .get_id_by_uuid(hotel_uuid)
            .await
            .inspect_err(|e| error!("Failed to get hotel id by uuid {}: {}", hotel_uuid, e))?
        {
            Ok(self
                .hotel_rating_repository
                .get_comments_by_hotel_id(hotel_id)
                .await
                .inspect_err(|e| {
                    error!("Failed to get comments by hotel id {}: {}", hotel_id, e)
                })?)
        } else {
            Err(HotelRatingServiceError::InvalidHotelUuid(hotel_uuid))
        }
    }

    #[instrument(skip(self, text))]
    async fn add_comment(
        &self,
        hotel_uuid: Uuid,
        user_id: UserId,
        rating: Rating,
        text: String,
    ) -> Result<(), HotelRatingServiceError> {
        if let Some(hotel_id) = self
            .hotel_repository
            .get_id_by_uuid(hotel_uuid)
            .await
            .inspect_err(|e| error!("Failed to get hotel id by uuid {}: {}", hotel_uuid, e))?
        {
            let quota = self
                .get_hotel_comment_quota(hotel_uuid, user_id)
                .await
                .inspect_err(|e| {
                    error!(
                        "Failed to get hotel comment quota for hotel uuid {} user id {}: {}",
                        hotel_uuid, user_id, e
                    )
                })?;
            let used = self
                .get_current_comment_count(hotel_uuid, user_id)
                .await
                .inspect_err(|e| {
                    error!(
                        "Failed to get hotel current comment count for hotel uuid {} user id {}: {}",
                        hotel_uuid, user_id, e
                    )
                })?;

            if used >= quota {
                return Err(HotelRatingServiceError::NoCommentsQuotaLeft(
                    hotel_uuid, quota,
                ));
            }

            if text.len() > HOTEL_MAX_COMMENT_LENGTH {
                return Err(HotelRatingServiceError::CommentLengthExceed {
                    limit: HOTEL_MAX_COMMENT_LENGTH,
                    actual: text.len(),
                });
            }

            let mut rating = HotelRating::new(None, user_id, hotel_id, Self::now(), rating, text);

            self.hotel_rating_repository
                .save(&mut rating)
                .await
                .inspect_err(|e| error!("Failed to save hotel rating: {}", e))?;

            Ok(())
        } else {
            Err(HotelRatingServiceError::InvalidHotelUuid(hotel_uuid))
        }
    }
}
