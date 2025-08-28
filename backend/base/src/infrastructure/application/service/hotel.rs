use crate::HOTEL_MAX_BOOKING_DAYS;
use crate::application::commands::hotel::{
    HotelInfoQuery, HotelOrderInfoQuery, HotelQuery, NewCommentCommand, QuotaQuery,
};
use crate::application::service::hotel::{
    HotelCommentDTO, HotelCommentQuotaDTO, HotelDetailInfoDTO, HotelGeneralInfoDTO,
    HotelRoomDetailInfoDTO, HotelService, HotelServiceError,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::hotel::{HotelDateRange, Rating};
use crate::domain::model::session::SessionId;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::user::UserRepository;
use crate::domain::service::hotel_booking::HotelBookingService;
use crate::domain::service::hotel_query::{HotelQueryError, HotelQueryService};
use crate::domain::service::hotel_rating::{HotelRatingService, HotelRatingServiceError};
use crate::domain::service::session::SessionManagerService;
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct HotelServiceImpl<HRS, HQS, HBS, HR, UR, SS>
where
    HRS: HotelRatingService,
    HQS: HotelQueryService,
    HBS: HotelBookingService,
    HR: HotelRepository,
    UR: UserRepository,
    SS: SessionManagerService,
{
    hotel_rating_service: Arc<HRS>,
    hotel_query_service: Arc<HQS>,
    hotel_booking_service: Arc<HBS>,
    hotel_repository: Arc<HR>,
    user_repository: Arc<UR>,
    session_manager: Arc<SS>,
}

impl<HRS, HQS, HBS, HR, UR, SS> HotelServiceImpl<HRS, HQS, HBS, HR, UR, SS>
where
    HRS: HotelRatingService,
    HQS: HotelQueryService,
    HBS: HotelBookingService,
    HR: HotelRepository,
    UR: UserRepository,
    SS: SessionManagerService,
{
    pub fn new(
        hotel_rating_service: Arc<HRS>,
        hotel_query_service: Arc<HQS>,
        hotel_booking_service: Arc<HBS>,
        hotel_repository: Arc<HR>,
        user_repository: Arc<UR>,
        session_manager: Arc<SS>,
    ) -> Self {
        HotelServiceImpl {
            hotel_rating_service,
            hotel_query_service,
            hotel_booking_service,
            hotel_repository,
            user_repository,
            session_manager,
        }
    }
}

#[async_trait]
impl<HRS, HQS, HBS, HR, UR, SS> HotelService for HotelServiceImpl<HRS, HQS, HBS, HR, UR, SS>
where
    HRS: HotelRatingService,
    HQS: HotelQueryService,
    HBS: HotelBookingService,
    HR: HotelRepository,
    UR: UserRepository,
    SS: SessionManagerService,
{
    #[instrument(skip(self))]
    async fn get_quota(
        &self,
        query: QuotaQuery,
    ) -> Result<HotelCommentQuotaDTO, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(query.session_id.as_str())
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|e| {
                error!("Failed to get user ID by session: {:?}", e);
                GeneralError::InternalServerError
            })?
            .ok_or(GeneralError::InvalidSessionId)?;

        let quota = self
            .hotel_rating_service
            .get_hotel_comment_quota(query.hotel_id, user_id)
            .await
            .map_err(|e| {
                error!("Failed to get hotel comment quota: {:?}", e);
                GeneralError::InternalServerError
            })?;

        let used = self
            .hotel_rating_service
            .get_current_comment_count(query.hotel_id, user_id)
            .await
            .map_err(|e| {
                error!("Failed to get hotel comment count: {:?}", e);
                GeneralError::InternalServerError
            })?;

        Ok(HotelCommentQuotaDTO { quota, used })
    }

    async fn new_comment(
        &self,
        command: NewCommentCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(command.session_id.as_str())
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|e| {
                error!("Failed to get user ID by session: {:?}", e);
                GeneralError::InternalServerError
            })?
            .ok_or(GeneralError::InvalidSessionId)?;

        let rating = Rating::try_from(
            Decimal::from_f64(command.rating)
                .ok_or(HotelServiceError::InvalidRating(command.rating))?,
        )
        .map_err(|_for_super_earth| HotelServiceError::InvalidRating(command.rating))?;

        self.hotel_rating_service
            .add_comment(command.hotel_id, user_id, rating, command.comment)
            .await
            .map_err(|e| match e {
                HotelRatingServiceError::InvalidHotelUuid(uuid) => Box::new(GeneralError::NotFound(
                    format!("Invalid hotel uuid: {}", uuid),
                ))
                    as Box<dyn ApplicationError>,
                HotelRatingServiceError::NoCommentsQuotaLeft(_, _) => {
                    Box::new(HotelServiceError::CommentCountExceed) as Box<dyn ApplicationError>
                }
                HotelRatingServiceError::CommentLengthExceed { limit, actual } => {
                    Box::new(HotelServiceError::CommentLengthExceed { limit, actual })
                        as Box<dyn ApplicationError>
                }
                e => {
                    error!("Failed to add hotel comment: {:?}", e);
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                }
            })?;

        Ok(())
    }

    async fn query_hotels(
        &self,
        query: HotelQuery,
    ) -> Result<Vec<HotelGeneralInfoDTO>, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(query.session_id.as_str())
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;

        self.session_manager
            .get_session(session_id)
            .await
            .map_err(|e| {
                error!("Failed to get session: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let date_range = match (query.begin_date, query.end_date) {
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => {
                return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                    "Both dates must be specified or none".into(),
                )));
            }
            (Some(begin), Some(end)) => {
                if end <= begin {
                    return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                        "End date must be after begin date".into(),
                    )));
                }

                let duration = end.signed_duration_since(begin).num_days();
                if duration > HOTEL_MAX_BOOKING_DAYS as i64 {
                    return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                        format!("Stay cannot exceed {} days", HOTEL_MAX_BOOKING_DAYS),
                    )));
                }

                match HotelDateRange::new(begin, end) {
                    Ok(range) => Some(range),
                    Err(e) => {
                        return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                            e.to_string(),
                        )));
                    }
                }
            }
        };

        let hotel_infos = self
            .hotel_query_service
            .query_hotels(
                &query.target,
                &query.target_type,
                query.search.as_deref(),
                date_range.as_ref(),
            )
            .await
            .map_err(|e| match e {
                HotelQueryError::TargetNotFound(target) => {
                    Box::new(HotelServiceError::TargetNotFound(target)) as Box<dyn ApplicationError>
                }
                HotelQueryError::InvalidDateRange(msg) => {
                    Box::new(HotelServiceError::InvalidDateRangeMessage(msg))
                        as Box<dyn ApplicationError>
                }
                _ => {
                    error!("Failed to query hotels: {:?}", e);
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                }
            })?;

        Ok(hotel_infos)
    }

    async fn query_hotel_info(
        &self,
        query: HotelInfoQuery,
    ) -> Result<HotelDetailInfoDTO, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(query.session_id.as_str())
            .map_err(|_| GeneralError::InvalidSessionId)?;

        self.session_manager
            .get_session(session_id)
            .await
            .map_err(|e| {
                error!("Failed to get session: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let hotel_id = self
            .hotel_repository
            .get_id_by_uuid(query.hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to get hotel id by uuid: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::NotFound(format!(
                "Invalid hotel uuid: {}",
                query.hotel_id
            ))) as Box<dyn ApplicationError>)?;

        let hotel = self
            .hotel_repository
            .find(hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to find hotel: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(GeneralError::InternalServerError)
            .inspect_err(|_for_super_earth| {
                error!("inconsistent state: hotel id {} not found, but get_id_by_uuid({}) returned the id", hotel_id, query.hotel_id);
            })?;

        let comments = self
            .hotel_rating_service
            .get_comments(query.hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to get hotel comments: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let mut comment_dtos = Vec::with_capacity(comments.len());
        for c in comments {
            let user_name =
                match self.user_repository.find(c.user_id()).await {
                    Ok(Some(user)) => user.username().to_string(),
                    Ok(None) => {
                        error!("User not found: {}", u64::from(c.user_id()));
                        return Err(Box::new(GeneralError::InternalServerError)
                            as Box<dyn ApplicationError>);
                    }
                    Err(e) => {
                        error!("Failed to find user {}: {:?}", u64::from(c.user_id()), e);
                        return Err(Box::new(GeneralError::InternalServerError)
                            as Box<dyn ApplicationError>);
                    }
                };

            comment_dtos.push(HotelCommentDTO {
                user_name,
                comment_time: c.time().to_rfc3339(),
                rating: Decimal::from(c.rating()).to_f64().unwrap_or(0.0),
                comment: c.text().to_string(),
            });
        }

        let hotel_rating = self
            .hotel_rating_service
            .get_hotel_rating(query.hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to get hotel rating: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let picture = if hotel.images().is_empty() {
            None
        } else {
            Some(
                hotel
                    .images()
                    .iter()
                    .map(|img_uuid| format!("/resource/hotel/images/{}", img_uuid))
                    .collect(),
            )
        };

        Ok(HotelDetailInfoDTO {
            hotel_id: query.hotel_id.to_string(),
            name: hotel.name().to_string(),
            address: hotel.address().to_string(),
            phone: hotel.phone().clone(),
            info: hotel.info().clone(),
            picture,
            rating: Decimal::from(hotel_rating).to_f64().unwrap_or(0.0),
            rating_count: hotel.total_rating_count(),
            total_bookings: hotel.total_booking_count(),
            comments: comment_dtos,
        })
    }

    async fn query_hotel_order_info(
        &self,
        query: HotelOrderInfoQuery,
    ) -> Result<HashMap<String, HotelRoomDetailInfoDTO>, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(query.session_id.as_str())
            .map_err(|_| GeneralError::InvalidSessionId)?;

        self.session_manager
            .get_session(session_id)
            .await
            .map_err(|e| {
                error!("Failed to get session: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let hotel_id = self
            .hotel_repository
            .get_id_by_uuid(query.hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to get hotel id by uuid: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::NotFound(format!(
                "Invalid hotel uuid: {}",
                query.hotel_id
            ))) as Box<dyn ApplicationError>)?;

        let hotel = self
            .hotel_repository
            .find(hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to find hotel: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(GeneralError::InternalServerError).inspect_err(|_for_super_earth| {
            error!("inconsistent state: hotel id {} not found, but get_id_by_uuid({}) returned the id", hotel_id, query.hotel_id);
        })?;

        let date_range = match (query.begin_date, query.end_date) {
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => {
                return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                    "Both dates must be specified or none".into(),
                )));
            }
            (Some(begin), Some(end)) => {
                if end <= begin {
                    return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                        "End date must be after begin date".into(),
                    )));
                }

                let duration = end.signed_duration_since(begin).num_days();
                if duration > HOTEL_MAX_BOOKING_DAYS as i64 {
                    return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                        format!("Stay cannot exceed {} days", HOTEL_MAX_BOOKING_DAYS),
                    )));
                }

                match HotelDateRange::new(begin, end) {
                    Ok(range) => Some(range),
                    Err(e) => {
                        return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                            e.to_string(),
                        )));
                    }
                }
            }
        };

        let mut result = HashMap::new();

        let available_rooms = if let Some(ref range) = date_range {
            self.hotel_booking_service
                .get_available_room(hotel_id, *range)
                .await
                .map_err(|e| {
                    error!("Failed to get available rooms: {:?}", e);
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                })?
        } else {
            HashMap::new()
        };

        for room_type in hotel.room_type_list() {
            let type_name = room_type.type_name().clone();
            let capacity = room_type.capacity();
            let room_type_id = room_type.get_id();

            let remain_count = if date_range.is_some() {
                if let Some(room_id) = room_type_id {
                    available_rooms
                        .get(&room_id)
                        .map(|status| status.remain_count)
                        .unwrap_or(0)
                } else {
                    capacity
                }
            } else {
                capacity
            };

            let price = room_type.price().to_f64().unwrap_or(0.0);

            result.insert(
                type_name,
                HotelRoomDetailInfoDTO {
                    capacity,
                    remain_count,
                    price,
                },
            );
        }

        Ok(result)
    }
}



// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::Arc;
//     use chrono::{Utc, Duration};
//     use crate::domain::model::hotel::{Hotel, HotelId};
//     
//     use crate::domain::service::mock::session::MockSessionManagerService;
//     use crate::domain::service::mock::hotel_rating::MockHotelRatingService;
//     use crate::domain::service::mock::hotel_query::MockHotelQueryService;
//     
//     use crate::domain::repository::mock::user::MockUserRepository;
//     use crate::domain::repository::mock::hotel::MockHotelRepository;
// 
//     // ================= get_quota =================
//     #[tokio::test]
//     async fn test_get_quota_success() {
//         let mut session = MockSessionManagerService::new();
//         session.expect_get_user_id_by_session()
//             .returning(|_| Ok(Some(1.into())));
// 
//         let mut rating = MockHotelRatingService::new();
//         rating.expect_get_hotel_comment_quota().returning(|_, _| Ok(5));
//         rating.expect_get_current_comment_count().returning(|_, _| Ok(2));
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(rating),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(MockHotelRepository::new()),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let query = QuotaQuery { session_id: "valid".into(), hotel_id: "h1".into() };
//         let result = service.get_quota(query).await.unwrap();
//         assert_eq!(result.quota, 5);
//         assert_eq!(result.used, 2);
//     }
// 
//     #[tokio::test]
//     async fn test_get_quota_invalid_session() {
//         let mut session = MockSessionManagerService::new();
//         session.expect_get_user_id_by_session()
//             .returning(|_| Ok(None));
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(MockHotelRatingService::new()),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(MockHotelRepository::new()),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let query = QuotaQuery { session_id: "invalid".into(), hotel_id: "h1".into() };
//         let err = service.get_quota(query).await.unwrap_err();
//         assert!(err.to_string().contains("InvalidSessionId"));
//     }
// 
//     // ================= new_comment =================
//     #[tokio::test]
//     async fn test_new_comment_success() {
//         let mut session = MockSessionManagerService::new();
//         session.expect_get_user_id_by_session()
//             .returning(|_| Ok(Some(2.into())));
// 
//         let mut rating = MockHotelRatingService::new();
//         rating.expect_add_comment()
//             .returning(|_, _, _, _, _| Ok(()));
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(rating),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(MockHotelRepository::new()),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelCommentDTO {
//             session_id: "s1".into(),
//             hotel_id: "h1".into(),
//             rating: 4.5,
//             comment: "good".into(),
//         };
// 
//         assert!(service.new_comment(dto).await.is_ok());
//     }
// 
//     #[tokio::test]
//     async fn test_new_comment_invalid_rating() {
//         let session = MockSessionManagerService::new();
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(MockHotelRatingService::new()),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(MockHotelRepository::new()),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelCommentDTO {
//             session_id: "s1".into(),
//             hotel_id: "h1".into(),
//             rating: 6.0,
//             comment: "bad".into(),
//         };
// 
//         let err = service.new_comment(dto).await.unwrap_err();
//         assert!(matches!(err, HotelServiceError::InvalidRating));
//     }
// 
//     // ================= query_hotels =================
//     #[tokio::test]
//     async fn test_query_hotels_success() {
//         let mut session = MockSessionManagerService::new();
//         session.expect_get_user_id_by_session()
//             .returning(|_| Ok(Some(3.into())));
// 
//         let mut query_service = MockHotelQueryService::new();
//         query_service.expect_query_hotels()
//             .returning(|_, _| Ok(vec![
//                 HotelGeneralInfoDTO { id: "h1".into(), name: "Hotel1".into(), ..Default::default() }
//             ]));
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(MockHotelRatingService::new()),
//             Arc::new(query_service),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(MockHotelRepository::new()),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelQueryDTO {
//             session_id: "s1".into(),
//             begin_date: Utc::now(),
//             end_date: Utc::now() + Duration::days(1),
//             city_name: "Beijing".into(),
//         };
// 
//         let result = service.query_hotels(dto).await.unwrap();
//         assert_eq!(result.len(), 1);
//         assert_eq!(result[0].name, "Hotel1");
//     }
// 
//     #[tokio::test]
//     async fn test_query_hotels_invalid_date_range() {
//         let session = MockSessionManagerService::new();
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(MockHotelRatingService::new()),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(MockHotelRepository::new()),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelQueryDTO {
//             session_id: "s1".into(),
//             begin_date: Utc::now(),
//             end_date: Utc::now(),
//             city_name: "Beijing".into(),
//         };
// 
//         let err = service.query_hotels(dto).await.unwrap_err();
//         assert!(matches!(err, HotelServiceError::InvalidDateRangeMessage(_)));
//     }
// 
//     // ================= query_hotel_info =================
//     #[tokio::test]
//     async fn test_query_hotel_info_success() {
//         let mut session = MockSessionManagerService::new();
//         session.expect_get_user_id_by_session()
//             .returning(|_| Ok(Some(4.into())));
// 
//         let mut repo = MockHotelRepository::new();
//         repo.expect_get_id_by_uuid()
//             .returning(|_| Ok(Some(HotelId::from(1u64))));
//         repo.expect_get_by_id()
//             .returning(|_| Ok(Some(Hotel::new(HotelId::from(1u64), "HotelX".into(), "Addr".into()))));
// 
//         let mut rating = MockHotelRatingService::new();
//         rating.expect_get_comments()
//             .returning(|_, _| Ok(vec![HotelComment::new("user".into(), 4.0, "nice".into())]));
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(rating),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(repo),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelInfoQueryDTO { session_id: "s1".into(), hotel_id: "h1".into() };
//         let result = service.query_hotel_info(dto).await.unwrap();
//         assert_eq!(result.name, "HotelX");
//         assert_eq!(result.comments.len(), 1);
//     }
// 
//     #[tokio::test]
//     async fn test_query_hotel_info_not_found() {
//         let mut session = MockSessionManagerService::new();
//         session.expect_get_user_id_by_session()
//             .returning(|_| Ok(Some(5.into())));
// 
//         let mut repo = MockHotelRepository::new();
//         repo.expect_get_id_by_uuid()
//             .returning(|_| Ok(None));
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(MockHotelRatingService::new()),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(repo),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelInfoQueryDTO { session_id: "s1".into(), hotel_id: "h2".into() };
//         let err = service.query_hotel_info(dto).await.unwrap_err();
//         assert!(err.to_string().contains("Invalid hotel uuid"));
//     }
// 
//     // ================= query_hotel_order_info =================
//     #[tokio::test]
//     async fn test_query_hotel_order_info_success() {
//         let mut session = MockSessionManagerService::new();
//         session.expect_get_user_id_by_session()
//             .returning(|_| Ok(Some(6.into())));
// 
//         let mut booking = MockHotelBookingService::new();
//         booking.expect_get_available_room()
//             .returning(|_, _, _, _| Ok(10));
// 
//         let mut repo = MockHotelRepository::new();
//         repo.expect_get_rooms_by_hotel_id()
//             .returning(|_| Ok(vec![
//                 Room::new("Deluxe".into(), 100.0, 5),
//                 Room::new("Standard".into(), 80.0, 5),
//             ]));
// 
//         repo.expect_get_id_by_uuid()
//             .returning(|_| Ok(Some(HotelId::from(1u64))));
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(MockHotelRatingService::new()),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(booking),
//             Arc::new(repo),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelOrderInfoQueryDTO {
//             session_id: "s1".into(),
//             hotel_id: "h1".into(),
//             begin_date: Utc::now(),
//             end_date: Utc::now() + Duration::days(1),
//         };
// 
//         let result = service.query_hotel_order_info(dto).await.unwrap();
//         assert!(result.contains_key("Deluxe"));
//         assert!(result.contains_key("Standard"));
//     }
// 
//     #[tokio::test]
//     async fn test_query_hotel_order_info_invalid_date() {
//         let session = MockSessionManagerService::new();
// 
//         let service = HotelServiceImpl::new(
//             Arc::new(MockHotelRatingService::new()),
//             Arc::new(MockHotelQueryService::new()),
//             Arc::new(MockHotelBookingService::new()),
//             Arc::new(MockHotelRepository::new()),
//             Arc::new(MockUserRepository::new()),
//             Arc::new(session),
//         );
// 
//         let dto = HotelOrderInfoQueryDTO {
//             session_id: "s1".into(),
//             hotel_id: "h1".into(),
//             begin_date: Utc::now(),
//             end_date: Utc::now(),
//         };
// 
//         let err = service.query_hotel_order_info(dto).await.unwrap_err();
//         assert!(matches!(err, HotelServiceError::InvalidDateRangeMessage(_)));
//     }
// }
