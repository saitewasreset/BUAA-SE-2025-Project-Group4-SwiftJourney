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
                HotelRatingServiceError::InvalidHotelUuid(_) => {
                    Box::new(GeneralError::NotFound) as Box<dyn ApplicationError>
                }
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
            .ok_or(Box::new(GeneralError::NotFound) as Box<dyn ApplicationError>)?;

        let hotel = self
            .hotel_repository
            .find(hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to find hotel: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::NotFound) as Box<dyn ApplicationError>)?;

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
                    .map(|img_uuid| format!("/resource/images/{}", img_uuid))
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
            .ok_or(Box::new(GeneralError::NotFound) as Box<dyn ApplicationError>)?;

        let hotel = self
            .hotel_repository
            .find(hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to find hotel: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::NotFound) as Box<dyn ApplicationError>)?;

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
