use crate::domain::model::hotel::{HotelRating, Rating};
use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::model::user::UserId;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::hotel_rating::HotelRatingRepository;
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::hotel_rating::{HotelRatingService, HotelRatingServiceError};
use crate::HOTEL_MAX_COMMENT_LENGTH;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::city::City;
    use crate::domain::model::hotel::{Hotel, HotelDateRange, HotelRating, HotelRoomType, Rating};
    use crate::domain::model::order::{BaseOrder, HotelOrder, OrderTimeInfo, PaymentInfo};
    use crate::domain::model::station::Station;
    use crate::domain::model::transaction::Transaction;
    use crate::domain::repository::mock::{
        hotel::MockHotelRepository, hotel_rating::MockHotelRatingRepository,
        order::MockOrderRepository,
    };
    use crate::HOTEL_MAX_COMMENT_LENGTH;
    use chrono::{Local, NaiveDate};
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use uuid::Uuid;

    fn build_service(
        hotel_repo: MockHotelRepository,
        rating_repo: MockHotelRatingRepository,
        order_repo: MockOrderRepository,
    ) -> HotelRatingServiceImpl<MockHotelRepository, MockHotelRatingRepository, MockOrderRepository>
    {
        HotelRatingServiceImpl::new(
            Arc::new(hotel_repo),
            Arc::new(rating_repo),
            Arc::new(order_repo),
        )
    }

    // ---------------- get_hotel_rating ----------------
    #[tokio::test]
    async fn test_get_hotel_rating_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();

        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let mut rating_repo = MockHotelRatingRepository::new();
        rating_repo
            .expect_get_hotel_rating()
            .returning(|_| Ok(Some(Rating::try_from(Decimal::new(45, 1)).unwrap())));

        let service = build_service(hotel_repo, rating_repo, MockOrderRepository::new());

        let rating = service.get_hotel_rating(hotel_uuid).await.unwrap();
        let expected = Rating::try_from(Decimal::new(45, 1)).unwrap(); // 4.5
        assert_eq!(rating, expected);
    }

    #[tokio::test]
    async fn test_get_hotel_rating_invalid_uuid() {
        let hotel_uuid = Uuid::new_v4();

        let hotel_repo = MockHotelRepository {
            hotel: None,
            hotel_id: None,
        };

        let mut hotel_rating_repo = MockHotelRatingRepository::new();
        hotel_rating_repo
            .expect_get_hotel_rating()
            .returning(|_| Ok(None));

        let service = build_service(hotel_repo, hotel_rating_repo, MockOrderRepository::new());

        let res = service.get_hotel_rating(hotel_uuid).await;
        assert!(matches!(
            res,
            Err(HotelRatingServiceError::InvalidHotelUuid(_))
        ));
    }

    // ---------------- get_hotel_comment_quota ----------------
    #[tokio::test]
    async fn test_get_hotel_comment_quota_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let user_id = 1u64.into();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Completed,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_hotel_order_by_userid()
            .returning(move |_, _| {
                Ok(vec![HotelOrder::new(
                    base_order.clone(),
                    hotel_id,
                    room_type_id,
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
                    )
                    .unwrap(),
                )])
            });

        let service = build_service(hotel_repo, MockHotelRatingRepository::new(), order_repo);

        let quota = service
            .get_hotel_comment_quota(hotel_uuid, user_id)
            .await
            .unwrap();
        assert_eq!(quota, 1); // sum of completed orders
    }

    #[tokio::test]
    async fn test_get_hotel_comment_quota_invalid_uuid() {
        let invalid_uuid = Uuid::new_v4();

        let hotel_repo = MockHotelRepository {
            hotel: None,
            hotel_id: None,
        };

        let user_id = 1u64.into();

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_hotel_order_by_userid()
            .returning(|_, _| Ok(vec![]));

        let service = build_service(hotel_repo, MockHotelRatingRepository::new(), order_repo);

        let res = service.get_hotel_comment_quota(invalid_uuid, user_id).await;
        assert!(matches!(
            res,
            Err(HotelRatingServiceError::InvalidHotelUuid(_))
        ));
    }

    // ---------------- get_current_comment_count ----------------
    #[tokio::test]
    async fn test_get_current_comment_count_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let user_id = 1u64.into();

        let mut rating_repo = MockHotelRatingRepository::new();
        rating_repo
            .expect_get_comments_by_user_id()
            .returning(move |_| {
                Ok(vec![HotelRating::new(
                    None,
                    user_id,
                    hotel_id,
                    DateTimeWithTimeZone::from(Local::now()),
                    Rating::try_from(Decimal::new(45, 1)).unwrap(),
                    "Good".to_string(),
                )])
            });

        let service = build_service(hotel_repo, rating_repo, MockOrderRepository::new());

        let count = service
            .get_current_comment_count(hotel_uuid, user_id)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_current_comment_count_invalid_uuid() {
        let invalid_uuid = Uuid::new_v4();

        let hotel_repo = MockHotelRepository {
            hotel: None,
            hotel_id: None, // 返回 None 表示找不到酒店
        };

        let hotel_rating_repo = MockHotelRatingRepository::new();
        let order_repo = MockOrderRepository::new();

        let service = build_service(hotel_repo, hotel_rating_repo, order_repo);

        let user_id = 1u64.into();
        let res = service
            .get_current_comment_count(invalid_uuid, user_id)
            .await;

        assert!(matches!(
            res,
            Err(HotelRatingServiceError::InvalidHotelUuid(_))
        ));
    }

    // ---------------- get_comments ----------------
    #[tokio::test]
    async fn test_get_comments_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let user_id = 1u64.into();

        let mut rating_repo = MockHotelRatingRepository::new();
        rating_repo
            .expect_get_comments_by_hotel_id()
            .returning(move |_| {
                Ok(vec![HotelRating::new(
                    None,
                    user_id,
                    1u64.into(),
                    DateTimeWithTimeZone::from(Local::now()),
                    Rating::try_from(Decimal::new(45, 1)).unwrap(),
                    "Nice".to_string(),
                )])
            });

        let service = build_service(hotel_repo, rating_repo, MockOrderRepository::new());

        let comments = service.get_comments(hotel_uuid).await.unwrap();
        assert_eq!(comments.len(), 1);
    }

    #[tokio::test]
    async fn test_get_comments_invalid_uuid() {
        let invalid_uuid = Uuid::new_v4();

        let hotel_repo = MockHotelRepository {
            hotel: None,
            hotel_id: None, // 返回 None 表示找不到酒店
        };

        let hotel_rating_repo = MockHotelRatingRepository::new();

        let service = build_service(hotel_repo, hotel_rating_repo, MockOrderRepository::new());

        let res = service.get_comments(invalid_uuid).await;

        assert!(matches!(
            res,
            Err(HotelRatingServiceError::InvalidHotelUuid(_))
        ));
    }

    // ---------------- add_comment ----------------
    #[tokio::test]
    async fn test_add_comment_success() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let user_id = 1u64.into();
        let hotel_rating_id = 1u64.into();

        let mut rating_repo = MockHotelRatingRepository::new();
        rating_repo
            .expect_get_comments_by_user_id()
            .returning(move |_| Ok(vec![]));
        rating_repo
            .expect_save()
            .returning(move |_| Ok(hotel_rating_id));

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Completed,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_hotel_order_by_userid()
            .returning(move |_, _| {
                Ok(vec![HotelOrder::new(
                    base_order.clone(),
                    hotel_id,
                    room_type_id,
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
                    )
                    .unwrap(),
                )])
            });

        let service = build_service(hotel_repo, rating_repo, order_repo);

        let res = service
            .add_comment(
                hotel_uuid,
                user_id,
                Rating::try_from(Decimal::new(45, 1)).unwrap(),
                "Nice stay".to_string(),
            )
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_add_comment_no_quota() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let user_id = 1u64.into();

        let mut rating_repo = MockHotelRatingRepository::new();
        rating_repo
            .expect_get_comments_by_user_id()
            .returning(move |_| Ok(vec![]));

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_hotel_order_by_userid()
            .returning(|_, _| Ok(vec![])); // quota=0

        let service = build_service(hotel_repo, rating_repo, order_repo);

        let res = service
            .add_comment(
                hotel_uuid,
                user_id,
                Rating::try_from(Decimal::new(45, 1)).unwrap(),
                "Nice stay".to_string(),
            )
            .await;
        assert!(matches!(
            res,
            Err(HotelRatingServiceError::NoCommentsQuotaLeft(_, 0))
        ));
    }

    #[tokio::test]
    async fn test_add_comment_text_too_long() {
        let hotel_uuid = Uuid::new_v4();
        let hotel_id = 1u64.into();
        let room_type_id = 101u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(1u64.into()),
            hotel_uuid,
            "日升大酒店".to_string(),
            City::new(
                Some(1u64.into()),
                "北京".to_string().into(),
                "北京市".to_string().into(),
            ),
            Station::new(Some(1u64.into()), "北京南".to_string(), 1u64.into()),
            "日升路".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![HotelRoomType::new(
                Some(room_type_id),
                Some(hotel_id),
                "大床房".to_string(),
                2,
                Decimal::new(1000, 2),
            )],
            "日升全程为您服务".to_string(),
        );
        let hotel_repo = MockHotelRepository {
            hotel: Some(hotel.clone()),
            hotel_id: Some(hotel_id),
        };

        let text = "a".repeat(HOTEL_MAX_COMMENT_LENGTH + 1);
        let user_id = 1u64.into();

        let mut rating_repo = MockHotelRatingRepository::new();
        rating_repo
            .expect_get_comments_by_user_id()
            .returning(move |_| Ok(vec![]));

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Completed,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let mut order_repo = MockOrderRepository::new();
        order_repo
            .expect_find_hotel_order_by_userid()
            .returning(move |_, _| {
                Ok(vec![HotelOrder::new(
                    base_order.clone(),
                    hotel_id,
                    room_type_id,
                    HotelDateRange::new(
                        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2025, 9, 3).unwrap(),
                    )
                    .unwrap(),
                )])
            });

        let service = build_service(hotel_repo, rating_repo, order_repo);

        let res = service
            .add_comment(
                hotel_uuid,
                user_id,
                Rating::try_from(Decimal::new(45, 1)).unwrap(),
                text,
            )
            .await;
        assert!(matches!(
            res,
            Err(HotelRatingServiceError::CommentLengthExceed { .. })
        ));
    }
}
