use crate::application::commands::hotel::{HotelQuery, NewCommentCommand, QuotaQuery};
use crate::application::service::hotel::{
    HotelCommentQuotaDTO, HotelGeneralInfoDTO, HotelService, HotelServiceError,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::hotel::{HotelDateRange, Rating};
use crate::domain::model::session::SessionId;
use crate::domain::service::hotel_query::{HotelQueryError, HotelQueryService};
use crate::domain::service::hotel_rating::{HotelRatingService, HotelRatingServiceError};
use crate::domain::service::session::SessionManagerService;
use crate::HOTEL_MAX_BOOKING_DAYS;
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct HotelServiceImpl<HRS, HQS, SS>
where
    HRS: HotelRatingService,
    HQS: HotelQueryService,
    SS: SessionManagerService,
{
    hotel_rating_service: Arc<HRS>,
    hotel_query_service: Arc<HQS>,
    session_manager: Arc<SS>,
}

impl<HRS, HQS, SS> HotelServiceImpl<HRS, HQS, SS>
where
    HRS: HotelRatingService,
    HQS: HotelQueryService,
    SS: SessionManagerService,
{
    pub fn new(
        hotel_rating_service: Arc<HRS>,
        hotel_query_service: Arc<HQS>,
        session_manager: Arc<SS>,
    ) -> Self {
        HotelServiceImpl {
            hotel_rating_service,
            hotel_query_service,
            session_manager,
        }
    }
}

#[async_trait]
impl<HRS, HQS, SS> HotelService for HotelServiceImpl<HRS, HQS, SS>
where
    HRS: HotelRatingService,
    HQS: HotelQueryService,
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
                    return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(format!(
                        "Stay cannot exceed {} days",
                        HOTEL_MAX_BOOKING_DAYS
                    ))));
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
}
