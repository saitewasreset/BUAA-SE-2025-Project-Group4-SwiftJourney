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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::commands::hotel::{HotelQuery, TargetType};
    use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
    use crate::domain::model::hotel::{Hotel, HotelId};
    use crate::domain::model::hotel::{HotelRoomStatus, HotelRoomType, HotelRoomTypeId};
    use crate::domain::model::session::Session;
    use crate::domain::model::station::Station;
    use crate::domain::model::user::{User, UserId};
    use crate::domain::service::hotel_booking::HotelBookingServiceError;
    use crate::domain::service::hotel_rating::HotelRatingServiceError;
    use crate::domain::{Repository, RepositoryError};
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    // -------- Session stubs --------
    struct SessOk;
    #[async_trait]
    impl SessionManagerService for SessOk {
        async fn create_session(&self, _user_id: UserId) -> Result<Session, RepositoryError> {
            Err(RepositoryError::Db(anyhow::anyhow!("not used")))
        }
        async fn delete_session(&self, _session: Session) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn get_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<Session>, RepositoryError> {
            let now = Utc::now();
            Ok(Some(Session::new(1u64.into(), now, now)))
        }
        async fn get_user_id_by_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<UserId>, RepositoryError> {
            Ok(Some(1u64.into()))
        }
        async fn verify_session_id(&self, _session_id_str: &str) -> Result<bool, RepositoryError> {
            Ok(true)
        }
    }

    struct SessUnused; // for cases where parsing fails before calling service
    #[async_trait]
    impl SessionManagerService for SessUnused {
        async fn create_session(&self, _user_id: UserId) -> Result<Session, RepositoryError> {
            unreachable!()
        }
        async fn delete_session(&self, _session: Session) -> Result<(), RepositoryError> {
            unreachable!()
        }
        async fn get_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<Session>, RepositoryError> {
            unreachable!()
        }
        async fn get_user_id_by_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<UserId>, RepositoryError> {
            unreachable!()
        }
        async fn verify_session_id(&self, _session_id_str: &str) -> Result<bool, RepositoryError> {
            unreachable!()
        }
    }

    // -------- Rating service stub --------
    enum AddMode {
        Ok,
        InvalidUuid,
        NoQuota,
        LengthExceed,
    }
    struct RatingSvc {
        add_mode: AddMode,
        quota: i32,
        used: i32,
    }
    #[async_trait]
    impl HotelRatingService for RatingSvc {
        async fn get_hotel_rating(
            &self,
            _hotel_uuid: Uuid,
        ) -> Result<Rating, HotelRatingServiceError> {
            Ok(Rating::default())
        }
        async fn get_hotel_comment_quota(
            &self,
            _hotel_uuid: Uuid,
            _user_id: UserId,
        ) -> Result<i32, HotelRatingServiceError> {
            Ok(self.quota)
        }
        async fn get_current_comment_count(
            &self,
            _hotel_uuid: Uuid,
            _user_id: UserId,
        ) -> Result<i32, HotelRatingServiceError> {
            Ok(self.used)
        }
        async fn get_comments(
            &self,
            _hotel_uuid: Uuid,
        ) -> Result<Vec<crate::domain::model::hotel::HotelRating>, HotelRatingServiceError>
        {
            Ok(vec![])
        }
        async fn add_comment(
            &self,
            hotel_uuid: Uuid,
            _user_id: UserId,
            _rating: Rating,
            _text: String,
        ) -> Result<(), HotelRatingServiceError> {
            match self.add_mode {
                AddMode::Ok => Ok(()),
                AddMode::InvalidUuid => Err(HotelRatingServiceError::InvalidHotelUuid(hotel_uuid)),
                AddMode::NoQuota => Err(HotelRatingServiceError::NoCommentsQuotaLeft(
                    hotel_uuid, self.quota,
                )),
                AddMode::LengthExceed => Err(HotelRatingServiceError::CommentLengthExceed {
                    limit: 10,
                    actual: 20,
                }),
            }
        }
    }

    // -------- Query service stub --------
    struct QuerySvcOk;
    #[async_trait]
    impl HotelQueryService for QuerySvcOk {
        async fn find_hotels_by_target(
            &self,
            _target: &str,
            _target_type: &TargetType,
            _search_term: Option<&str>,
        ) -> Result<Vec<Hotel>, HotelQueryError> {
            Ok(vec![])
        }
        async fn calculate_minimum_prices(
            &self,
            _hotels: &[Hotel],
            _date_range: Option<&HotelDateRange>,
        ) -> Result<std::collections::HashMap<HotelId, rust_decimal::Decimal>, HotelQueryError>
        {
            Ok(HashMap::new())
        }
        async fn query_hotels(
            &self,
            target: &str,
            _target_type: &TargetType,
            _search_term: Option<&str>,
            _date_range: Option<&HotelDateRange>,
        ) -> Result<Vec<HotelGeneralInfoDTO>, HotelQueryError> {
            Ok(vec![HotelGeneralInfoDTO {
                hotel_id: Uuid::new_v4(),
                name: format!("Hotel@{}", target),
                picture: None,
                rating: 4.2,
                rating_count: 10,
                total_bookings: 30,
                price: 199.0,
                info: "nice".to_string(),
            }])
        }
    }

    // -------- Booking service stub --------
    struct BookingSvc;
    #[async_trait]
    impl HotelBookingService for BookingSvc {
        async fn get_available_room(
            &self,
            _hotel_id: HotelId,
            _booking_date_range: HotelDateRange,
        ) -> Result<
            std::collections::HashMap<
                crate::domain::model::hotel::HotelRoomTypeId,
                crate::domain::model::hotel::HotelRoomStatus,
            >,
            HotelBookingServiceError,
        > {
            Ok(HashMap::new())
        }
        async fn booking_hotel(&self, _order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
            Ok(())
        }
        async fn cancel_hotel(&self, _order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
            Ok(())
        }
        async fn booking_group(
            &self,
            _order_uuid_list: Vec<Uuid>,
            _atomic: bool,
        ) -> Result<Vec<crate::domain::model::order::HotelOrder>, HotelBookingServiceError>
        {
            Ok(vec![])
        }
    }

    // -------- Hotel repository stub --------
    struct HotelRepo;
    #[async_trait]
    impl HotelRepository for HotelRepo {
        async fn get_id_by_uuid(&self, _uuid: Uuid) -> Result<Option<HotelId>, RepositoryError> {
            Ok(None)
        }
        async fn find_by_uuid(&self, _uuid: Uuid) -> Result<Option<Hotel>, RepositoryError> {
            Ok(None)
        }
        async fn find_by_city(
            &self,
            _city_id: crate::domain::model::city::CityId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
        async fn find_by_station(
            &self,
            _station_id: crate::domain::model::station::StationId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
    }
    #[async_trait]
    impl Repository<Hotel> for HotelRepo {
        async fn find(&self, _id: HotelId) -> Result<Option<Hotel>, RepositoryError> {
            Ok(None)
        }
        async fn remove(&self, _aggregate: Hotel) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(&self, _aggregate: &mut Hotel) -> Result<HotelId, RepositoryError> {
            Ok(1u64.into())
        }
    }

    // -------- User repository stub --------
    struct UserRepo;
    #[async_trait]
    impl UserRepository for UserRepo {
        async fn find_by_phone(
            &self,
            _phone: crate::domain::model::user::Phone,
        ) -> Result<Option<User>, RepositoryError> {
            Ok(None)
        }
        async fn find_by_identity_card_id(
            &self,
            _identity_card_id: crate::domain::model::user::IdentityCardId,
        ) -> Result<Option<User>, RepositoryError> {
            Ok(None)
        }
        async fn remove_by_phone(
            &self,
            _phone: crate::domain::model::user::Phone,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }
    #[async_trait]
    impl Repository<User> for UserRepo {
        async fn find(&self, _id: UserId) -> Result<Option<User>, RepositoryError> {
            Ok(None)
        }
        async fn remove(&self, _aggregate: User) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(&self, _aggregate: &mut User) -> Result<UserId, RepositoryError> {
            Ok(1u64.into())
        }
    }

    // ---------------- Tests ----------------
    #[tokio::test]
    async fn get_quota_success() {
        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 5,
                used: 2,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );

        let q = QuotaQuery {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: Uuid::new_v4(),
        };
        let got = svc.get_quota(q).await.unwrap();
        assert_eq!(got.quota, 5);
        assert_eq!(got.used, 2);
    }

    #[tokio::test]
    async fn get_quota_invalid_session_id_format() {
        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 0,
                used: 0,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessUnused),
        );

        let q = QuotaQuery {
            session_id: "not-a-uuid".into(),
            hotel_id: Uuid::new_v4(),
        };
        match svc.get_quota(q).await {
            Ok(v) => panic!("expected error, got {:?}", v.quota),
            Err(err) => assert!(err.to_string().contains("invalid session id")),
        }
    }

    #[tokio::test]
    async fn new_comment_success() {
        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 5,
                used: 1,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let cmd = NewCommentCommand {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: Uuid::new_v4(),
            rating: 4.5,
            comment: "Great!".to_string(),
        };
        assert!(svc.new_comment(cmd).await.is_ok());
    }

    #[tokio::test]
    async fn new_comment_invalid_rating() {
        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 5,
                used: 1,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let cmd = NewCommentCommand {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: Uuid::new_v4(),
            rating: 6.0,
            comment: "too high".to_string(),
        };
        let err = svc.new_comment(cmd).await.unwrap_err();
        assert!(err.to_string().contains("invalid rating"));
    }

    #[tokio::test]
    async fn new_comment_error_mappings() {
        // InvalidHotelUuid -> NotFound
        let svc1 = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::InvalidUuid,
                quota: 0,
                used: 0,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let cmd = NewCommentCommand {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: Uuid::new_v4(),
            rating: 3.0,
            comment: "x".into(),
        };
        let err = svc1.new_comment(cmd).await.unwrap_err();
        assert!(err.to_string().contains("Invalid hotel uuid"));

        // NoCommentsQuotaLeft -> CommentCountExceed
        let svc2 = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::NoQuota,
                quota: 1,
                used: 1,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let cmd = NewCommentCommand {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: Uuid::new_v4(),
            rating: 3.0,
            comment: "x".into(),
        };
        let err = svc2.new_comment(cmd).await.unwrap_err();
        assert!(err.to_string().contains("comment count exceed"));

        // CommentLengthExceed -> mapped accordingly
        let svc3 = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::LengthExceed,
                quota: 5,
                used: 0,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let cmd = NewCommentCommand {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: Uuid::new_v4(),
            rating: 3.0,
            comment: "xxxxxxxxxxxxxxxxxxxxxxxx".into(),
        };
        let err = svc3.new_comment(cmd).await.unwrap_err();
        assert!(err.to_string().contains("comment length exceed"));
    }

    #[tokio::test]
    async fn query_hotels_success_minimal() {
        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 0,
                used: 0,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let q = HotelQuery {
            session_id: Uuid::new_v4().to_string(),
            target: "Beijing".into(),
            target_type: TargetType::City,
            search: None,
            begin_date: None,
            end_date: None,
        };
        let list = svc.query_hotels(q).await.unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].name.contains("Beijing"));
    }

    #[tokio::test]
    async fn query_hotels_invalid_date_range_equal() {
        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 0,
                used: 0,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepo),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let today = chrono::Local::now().date_naive();
        let q = HotelQuery {
            session_id: Uuid::new_v4().to_string(),
            target: "BJ".into(),
            target_type: TargetType::City,
            search: None,
            begin_date: Some(today),
            end_date: Some(today),
        };
        match svc.query_hotels(q).await {
            Ok(v) => panic!("expected error, got {} items", v.len()),
            Err(err) => assert!(
                err.to_string()
                    .contains("End date must be after begin date")
            ),
        }
    }

    // -------- query_hotel_info --------
    struct HotelRepoInfoOk {
        hotel: Hotel,
        hid: HotelId,
    }
    #[async_trait]
    impl HotelRepository for HotelRepoInfoOk {
        async fn get_id_by_uuid(&self, _uuid: Uuid) -> Result<Option<HotelId>, RepositoryError> {
            Ok(Some(self.hid))
        }
        async fn find_by_uuid(&self, _uuid: Uuid) -> Result<Option<Hotel>, RepositoryError> {
            Ok(Some(self.hotel.clone()))
        }
        async fn find_by_city(
            &self,
            _city_id: crate::domain::model::city::CityId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
        async fn find_by_station(
            &self,
            _station_id: crate::domain::model::station::StationId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
    }
    #[async_trait]
    impl Repository<Hotel> for HotelRepoInfoOk {
        async fn find(&self, _id: HotelId) -> Result<Option<Hotel>, RepositoryError> {
            Ok(Some(self.hotel.clone()))
        }
        async fn remove(&self, _aggregate: Hotel) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(&self, _aggregate: &mut Hotel) -> Result<HotelId, RepositoryError> {
            Ok(self.hid)
        }
    }

    #[tokio::test]
    async fn query_hotel_info_success() {
        // build a minimal Hotel aggregate
        let city = City::new(
            Some(CityId::from(1u64)),
            CityName::from("C".to_string()),
            ProvinceName::from("P".to_string()),
        );
        let station = Station::new(Some(1u64.into()), "S".to_string(), CityId::from(1u64));
        let hotel_id: HotelId = 10u64.into();
        let hotel = Hotel::new_full_unchecked(
            Some(hotel_id),
            Uuid::new_v4(),
            "NiceHotel".to_string(),
            city,
            station,
            "Addr".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![],
            "Info".to_string(),
        );

        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 0,
                used: 0,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(BookingSvc),
            Arc::new(HotelRepoInfoOk {
                hotel: hotel.clone(),
                hid: hotel_id,
            }),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );

        let dto = HotelInfoQuery {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: hotel.uuid(),
        };
        let out = svc.query_hotel_info(dto).await.unwrap();
        assert_eq!(out.name, "NiceHotel");
        assert!(out.picture.is_none());
    }

    // -------- query_hotel_order_info --------
    struct BookingSvcMap {
        map: HashMap<HotelRoomTypeId, HotelRoomStatus>,
    }
    #[async_trait]
    impl HotelBookingService for BookingSvcMap {
        async fn get_available_room(
            &self,
            _hotel_id: HotelId,
            _booking_date_range: HotelDateRange,
        ) -> Result<HashMap<HotelRoomTypeId, HotelRoomStatus>, HotelBookingServiceError> {
            Ok(self.map.clone())
        }
        async fn booking_hotel(&self, _order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
            Ok(())
        }
        async fn cancel_hotel(&self, _order_uuid: Uuid) -> Result<(), HotelBookingServiceError> {
            Ok(())
        }
        async fn booking_group(
            &self,
            _order_uuid_list: Vec<Uuid>,
            _atomic: bool,
        ) -> Result<Vec<crate::domain::model::order::HotelOrder>, HotelBookingServiceError>
        {
            Ok(vec![])
        }
    }

    struct HotelRepoOrderOk {
        hotel: Hotel,
        hid: HotelId,
    }
    #[async_trait]
    impl HotelRepository for HotelRepoOrderOk {
        async fn get_id_by_uuid(&self, _uuid: Uuid) -> Result<Option<HotelId>, RepositoryError> {
            Ok(Some(self.hid))
        }
        async fn find_by_uuid(&self, _uuid: Uuid) -> Result<Option<Hotel>, RepositoryError> {
            Ok(Some(self.hotel.clone()))
        }
        async fn find_by_city(
            &self,
            _city_id: crate::domain::model::city::CityId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
        async fn find_by_station(
            &self,
            _station_id: crate::domain::model::station::StationId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
    }
    #[async_trait]
    impl Repository<Hotel> for HotelRepoOrderOk {
        async fn find(&self, _id: HotelId) -> Result<Option<Hotel>, RepositoryError> {
            Ok(Some(self.hotel.clone()))
        }
        async fn remove(&self, _aggregate: Hotel) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(&self, _aggregate: &mut Hotel) -> Result<HotelId, RepositoryError> {
            Ok(self.hid)
        }
    }

    #[tokio::test]
    async fn query_hotel_order_info_success() {
        let city = City::new(
            Some(CityId::from(1u64)),
            CityName::from("C".to_string()),
            ProvinceName::from("P".to_string()),
        );
        let station = Station::new(Some(1u64.into()), "S".to_string(), CityId::from(1u64));
        let hotel_id: HotelId = 20u64.into();
        let rt_with_id = HotelRoomType::new(
            Some(100u64.into()),
            Some(hotel_id),
            "Deluxe".to_string(),
            2,
            Decimal::from(100),
        );
        let rt_no_id = HotelRoomType::new(
            None,
            Some(hotel_id),
            "Standard".to_string(),
            3,
            Decimal::from(80),
        );
        let hotel = Hotel::new_full_unchecked(
            Some(hotel_id),
            Uuid::new_v4(),
            "H".to_string(),
            city,
            station,
            "Addr".to_string(),
            vec![],
            vec![],
            0,
            0,
            vec![rt_with_id.clone(), rt_no_id.clone()],
            "Info".to_string(),
        );
        let mut map = HashMap::new();
        map.insert(
            rt_with_id.get_id().unwrap(),
            HotelRoomStatus {
                capacity: 2,
                remain_count: 3,
                price: Decimal::from(100),
            },
        );
        let booking = BookingSvcMap { map };

        let svc = HotelServiceImpl::new(
            Arc::new(RatingSvc {
                add_mode: AddMode::Ok,
                quota: 0,
                used: 0,
            }),
            Arc::new(QuerySvcOk),
            Arc::new(booking),
            Arc::new(HotelRepoOrderOk {
                hotel: hotel.clone(),
                hid: hotel_id,
            }),
            Arc::new(UserRepo),
            Arc::new(SessOk),
        );
        let today = chrono::Local::now().date_naive();
        let q = HotelOrderInfoQuery {
            session_id: Uuid::new_v4().to_string(),
            hotel_id: hotel.uuid(),
            begin_date: Some(today),
            end_date: Some(today.succ_opt().unwrap()),
        };
        let out = svc.query_hotel_order_info(q).await.unwrap();
        assert_eq!(out.get("Deluxe").unwrap().remain_count, 3);
        assert_eq!(out.get("Standard").unwrap().remain_count, 3); // fallback to capacity when no id
    }
}
