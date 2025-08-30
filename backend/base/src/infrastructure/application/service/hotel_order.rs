use crate::application::commands::hotel_order::HotelOrderRequestDTO;
use crate::application::commands::hotel_order::HotelOrderRequestsDTO;
use crate::application::service::hotel::HotelServiceError;
use crate::application::service::hotel_order::HotelOrderService;
use crate::application::service::transaction::TransactionInfoDTO;
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::hotel::HotelDateRange;
use crate::domain::model::order::HotelOrder;
use crate::domain::model::order::{BaseOrder, Order, OrderStatus, OrderTimeInfo, PaymentInfo};
use crate::domain::model::session::SessionId;
use crate::domain::model::user::UserId;
use crate::domain::repository::hotel::HotelRepository;
use crate::domain::repository::order::OrderRepository;
use crate::domain::repository::personal_info::PersonalInfoRepository;
use crate::domain::service::hotel_booking::HotelBookingService;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::transaction::TransactionService;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, TimeZone};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

pub struct HotelOrderServiceImpl<HR, HBS, OR, TS, SMS, PIR> {
    hotel_repository: Arc<HR>,
    hotel_booking_service: Arc<HBS>,
    order_repository: Arc<OR>,
    transaction_service: Arc<TS>,
    session_manager: Arc<SMS>,
    personal_info_repository: Arc<PIR>,
}

impl<HR, HBS, OR, TS, SMS, PIR> HotelOrderServiceImpl<HR, HBS, OR, TS, SMS, PIR>
where
    HR: HotelRepository,
    HBS: HotelBookingService,
    OR: OrderRepository,
    TS: TransactionService,
    SMS: SessionManagerService,
    PIR: PersonalInfoRepository,
{
    pub fn new(
        hotel_repository: Arc<HR>,
        hotel_booking_service: Arc<HBS>,
        order_repository: Arc<OR>,
        transaction_service: Arc<TS>,
        session_manager: Arc<SMS>,
        personal_info_repository: Arc<PIR>,
    ) -> Self {
        Self {
            hotel_repository,
            hotel_booking_service,
            order_repository,
            transaction_service,
            session_manager,
            personal_info_repository,
        }
    }

    async fn validate_and_create_hotel_order(
        &self,
        dto: &HotelOrderRequestDTO,
        user_id: UserId,
    ) -> Result<Box<dyn Order>, Box<dyn ApplicationError>> {
        let hotel_uuid = Uuid::parse_str(&dto.hotel_id).map_err(|_| {
            Box::new(GeneralError::BadRequest(format!(
                "Invalid hotel id format: {}",
                dto.hotel_id
            ))) as Box<dyn ApplicationError>
        })?;
        let hotel_id = self
            .hotel_repository
            .get_id_by_uuid(hotel_uuid)
            .await
            .map_err(|e| {
                error!("Failed to get hotel id by uuid: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::NotFound(format!(
                "invalid hotel uuid: {}",
                hotel_uuid
            ))) as Box<dyn ApplicationError>)?;

        let hotel = self
            .hotel_repository
            .find(hotel_id)
            .await
            .map_err(|e| {
                error!("Failed to find hotel: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::NotFound(format!(
                "Invalid hotel uuid: {}",
                dto.hotel_id,
            ))) as Box<dyn ApplicationError>)?;

        let room_type = hotel
            .room_type_list()
            .iter()
            .find(|rt| rt.type_name() == &dto.room_type)
            .ok_or(Box::new(GeneralError::NotFound(format!(
                "Invalid hotel room type: {}",
                dto.room_type
            ))) as Box<dyn ApplicationError>)?;

        let room_type_id = room_type
            .get_id()
            .ok_or(Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>)?;

        let room_price_per_day = room_type.price();

        let date_range = match (dto.begin_date.as_ref(), dto.end_date.as_ref()) {
            (Some(begin), Some(end)) => {
                let begin_date = NaiveDate::parse_from_str(begin, "%Y-%m-%d").map_err(|_| {
                    Box::new(HotelServiceError::InvalidDateRangeMessage(
                        "Invalid date format".to_string(),
                    )) as Box<dyn ApplicationError>
                })?;
                let end_date = NaiveDate::parse_from_str(end, "%Y-%m-%d").map_err(|_| {
                    Box::new(HotelServiceError::InvalidDateRangeMessage(
                        "Invalid date format".to_string(),
                    )) as Box<dyn ApplicationError>
                })?;

                if end_date <= begin_date {
                    return Err(
                        Box::new(HotelServiceError::InvalidDateRange(begin_date, end_date))
                            as Box<dyn ApplicationError>,
                    );
                }

                let duration = end_date.signed_duration_since(begin_date).num_days();
                if duration > 7 {
                    return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                        "Stay cannot exceed 7 days".to_string(),
                    )) as Box<dyn ApplicationError>);
                }

                HotelDateRange::new(begin_date, end_date).map_err(|e| {
                    Box::new(HotelServiceError::InvalidDateRangeMessage(e.to_string()))
                        as Box<dyn ApplicationError>
                })?
            }
            _ => {
                return Err(Box::new(HotelServiceError::InvalidDateRangeMessage(
                    "Both begin date and end date must be provided".to_string(),
                )) as Box<dyn ApplicationError>);
            }
        };

        // 解析 UUID
        let personal_uuid = match Uuid::parse_str(&dto.personal_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                return Err(Box::new(GeneralError::BadRequest(format!(
                    "Invalid personal id format: {}",
                    dto.personal_id
                ))) as Box<dyn ApplicationError>);
            }
        };

        let personal_infos = self
            .personal_info_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|e| {
                error!("Database error when finding personal info: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let personal_info = personal_infos
            .into_iter()
            .find(|info| info.uuid() == personal_uuid)
            .ok_or(Box::new(GeneralError::NotFound(format!(
                "invalid personal info uuid: {}",
                personal_uuid
            ))) as Box<dyn ApplicationError>)?;

        let personal_info_id = personal_info
            .get_id()
            .ok_or(Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>)?;

        let days = date_range
            .end_date()
            .signed_duration_since(date_range.begin_date())
            .num_days() as u32;
        let room_price = room_price_per_day * Decimal::from(days);

        let amount = Decimal::from(dto.amount);

        if amount <= Decimal::ZERO {
            return Err(Box::new(GeneralError::BadRequest(
                "Amount must be greater than zero".to_string(),
            )) as Box<dyn ApplicationError>);
        }

        let order_uuid = Uuid::new_v4();
        let payment_info = PaymentInfo::new(None, None);

        let create_time: DateTimeWithTimeZone = chrono::Local::now().into();

        let begin_time: DateTimeWithTimeZone = {
            let local_time = chrono::Local
                .with_ymd_and_hms(
                    date_range.begin_date().year(),
                    date_range.begin_date().month(),
                    date_range.begin_date().day(),
                    14,
                    0,
                    0, // 14:00 入住
                )
                .single()
                .unwrap_or_else(chrono::Local::now);
            local_time.into()
        };

        let end_time: DateTimeWithTimeZone = {
            let local_time = chrono::Local
                .with_ymd_and_hms(
                    date_range.end_date().year(),
                    date_range.end_date().month(),
                    date_range.end_date().day(),
                    12,
                    0,
                    0, // 12:00 退房
                )
                .single()
                .unwrap_or_else(chrono::Local::now);
            local_time.into()
        };

        let order_time_info = OrderTimeInfo::new(create_time, begin_time, end_time);

        let base_order = BaseOrder::new(
            None,
            order_uuid,
            OrderStatus::Unpaid,
            order_time_info,
            room_price,
            amount,
            payment_info,
            personal_info_id,
        );

        let hotel_order = HotelOrder::new(base_order, hotel_id, room_type_id, date_range);

        Ok(Box::new(hotel_order))
    }

    #[instrument(skip(self))]
    pub async fn process_order_message(
        &self,
        transaction_id: Uuid,
        order_uuids: Vec<Uuid>,
        atomic: bool,
    ) -> Result<(), Box<dyn ApplicationError>> {
        info!(
            "Processing hotel orders for transaction: {}",
            transaction_id
        );

        let result = self
            .hotel_booking_service
            .booking_group(order_uuids.clone(), atomic)
            .await;

        match result {
            Ok(_) => {
                info!(
                    "Successfully processed hotel orders for transaction: {}",
                    transaction_id
                );
                Ok(())
            }
            Err(err) => {
                error!(
                    "Failed to process hotel orders for transaction {}: {:?}",
                    transaction_id, err
                );

                info!(
                    "Initiating automatic refund for failed transaction: {}",
                    transaction_id
                );

                let mut to_refund_orders: Vec<Box<dyn Order>> = Vec::new();

                for order_uuid in order_uuids.clone() {
                    match self
                        .order_repository
                        .find_hotel_order_by_uuid(order_uuid)
                        .await
                    {
                        Ok(Some(order)) => {
                            to_refund_orders.push(Box::new(order));
                        }
                        Ok(None) => {
                            error!("Order {} not found for refund", order_uuid);
                        }
                        Err(e) => {
                            error!("Error finding order {}: {:?}", order_uuid, e);
                        }
                    }
                }

                self.transaction_service
                    .refund_transaction(transaction_id, &to_refund_orders)
                    .await
                    .map_err(|e| {
                        error!("Failed to create refund transaction: {:?}", e);
                        Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                    })?;

                Err(Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>)
            }
        }
    }
}

#[async_trait]
impl<HR, HBS, OR, TS, SMS, PIR> HotelOrderService
    for HotelOrderServiceImpl<HR, HBS, OR, TS, SMS, PIR>
where
    HR: HotelRepository,
    HBS: HotelBookingService,
    OR: OrderRepository,
    TS: TransactionService,
    SMS: SessionManagerService,
    PIR: PersonalInfoRepository,
{
    #[instrument(skip(self, hotel_orders), fields(session_id = %session_id))]
    async fn process_hotel_orders(
        &self,
        session_id: String,
        hotel_orders: HotelOrderRequestsDTO,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        if hotel_orders.is_empty() {
            return Err(
                Box::new(GeneralError::BadRequest("Empty order list".to_string()))
                    as Box<dyn ApplicationError>,
            );
        }

        let session_id = SessionId::try_from(session_id.as_str())
            .map_err(|_| Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|e| {
                error!("Failed to get user id: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let mut orders: Vec<Box<dyn Order>> = Vec::new();
        for order_dto in &hotel_orders {
            let order = self
                .validate_and_create_hotel_order(order_dto, user_id)
                .await?;
            orders.push(order);
        }

        let total_amount = orders
            .iter()
            .map(|order| {
                (order.unit_price() * order.amount())
                    .to_f64()
                    .unwrap_or(0.0)
            })
            .sum::<f64>();

        let transaction_id = self
            .transaction_service
            .new_transaction(user_id, orders, true)
            .await
            .map_err(|e| {
                error!("Failed to create transaction: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        Ok(TransactionInfoDTO {
            transaction_id,
            amount: total_amount,
            status: "unpaid".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use std::collections::HashMap;

    use crate::domain::RepositoryError;
    use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
    use crate::domain::model::hotel::{
        Hotel, HotelDateRange, HotelId, HotelRoomStatus, HotelRoomType, HotelRoomTypeId,
    };
    use crate::domain::model::order::{BaseOrder, Order, OrderStatus, OrderTimeInfo, PaymentInfo};
    use crate::domain::model::personal_info::{PersonalInfo, PersonalInfoId};
    use crate::domain::model::session::Session;
    use crate::domain::model::station::{Station, StationId};
    use crate::domain::model::user::{IdentityCardId, RealName};
    use crate::domain::repository::hotel::HotelRepository;
    use crate::domain::repository::order::OrderRepository;
    use crate::domain::repository::personal_info::PersonalInfoRepository;
    use crate::domain::service::hotel_booking::{HotelBookingService, HotelBookingServiceError};
    use crate::domain::service::session::SessionManagerService;
    use crate::domain::service::transaction::{TransactionService, TransactionServiceError};

    // --------- 简易 Mock 实现 ---------
    struct MockHotelRepository {
        hotel: Option<Hotel>,
    }

    #[async_trait]
    impl HotelRepository for MockHotelRepository {
        async fn get_id_by_uuid(&self, uuid: Uuid) -> Result<Option<HotelId>, RepositoryError> {
            Ok(self
                .hotel
                .as_ref()
                .and_then(|h| if h.uuid() == uuid { h.get_id() } else { None }))
        }
        async fn find_by_uuid(&self, _uuid: Uuid) -> Result<Option<Hotel>, RepositoryError> {
            Ok(None)
        }
        async fn find_by_city(
            &self,
            _city_id: CityId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
        async fn find_by_station(
            &self,
            _station_id: StationId,
            _name_pattern: Option<&str>,
        ) -> Result<Vec<Hotel>, RepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl crate::domain::Repository<Hotel> for MockHotelRepository {
        async fn find(&self, id: HotelId) -> Result<Option<Hotel>, RepositoryError> {
            Ok(self.hotel.clone().filter(|h| h.get_id() == Some(id)))
        }
        async fn remove(&self, _aggregate: Hotel) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(&self, _aggregate: &mut Hotel) -> Result<HotelId, RepositoryError> {
            Ok(HotelId::from(1u64))
        }
    }

    struct MockHotelBookingService {
        pub ok: bool,
    }
    #[async_trait]
    impl HotelBookingService for MockHotelBookingService {
        async fn get_available_room(
            &self,
            _hotel_id: HotelId,
            _booking_date_range: HotelDateRange,
        ) -> Result<HashMap<HotelRoomTypeId, HotelRoomStatus>, HotelBookingServiceError> {
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
            if self.ok {
                Ok(vec![])
            } else {
                Err(HotelBookingServiceError::NoAvailableRoom(Uuid::new_v4()))
            }
        }
    }

    struct MockOrderRepository {
        pub hotel_order: Option<crate::domain::model::order::HotelOrder>,
    }
    #[async_trait]
    impl OrderRepository for MockOrderRepository {
        async fn find_train_order_by_uuid(
            &self,
            _order_uuid: Uuid,
        ) -> Result<Option<crate::domain::model::order::TrainOrder>, RepositoryError> {
            Ok(None)
        }
        async fn find_hotel_order_by_uuid(
            &self,
            _order_uuid: Uuid,
        ) -> Result<Option<crate::domain::model::order::HotelOrder>, RepositoryError> {
            Ok(self.hotel_order.clone())
        }
        async fn find_hotel_order_by_userid(
            &self,
            _user_id: crate::domain::model::user::UserId,
            _hotel_id: HotelId,
        ) -> Result<Vec<crate::domain::model::order::HotelOrder>, RepositoryError> {
            Ok(vec![])
        }
        async fn find_dish_order_by_uuid(
            &self,
            _order_uuid: Uuid,
        ) -> Result<Option<crate::domain::model::order::DishOrder>, RepositoryError> {
            Ok(None)
        }
        async fn find_takeaway_order_by_uuid(
            &self,
            _order_uuid: Uuid,
        ) -> Result<Option<crate::domain::model::order::TakeawayOrder>, RepositoryError> {
            Ok(None)
        }
        async fn load_all_active_orders(&self) -> Result<Vec<Box<dyn Order>>, RepositoryError> {
            Ok(vec![])
        }
        async fn update(&self, _order: Box<dyn Order>) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn get_route_info_train_order(
            &self,
            _order_id: crate::domain::model::order::OrderId,
            _train_schedule_id: crate::domain::model::train_schedule::TrainScheduleId,
        ) -> Result<
            (
                chrono::NaiveDate,
                Vec<crate::domain::repository::order::RouteInfo>,
            ),
            RepositoryError,
        > {
            Ok((chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), vec![]))
        }
        async fn get_route_info_takeaway_order(
            &self,
            _order_id: crate::domain::model::order::OrderId,
            _train_order_id: crate::domain::model::order::OrderId,
        ) -> Result<
            (
                chrono::NaiveDate,
                Vec<crate::domain::repository::order::RouteInfo>,
            ),
            RepositoryError,
        > {
            Ok((chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), vec![]))
        }
        async fn get_train_order_related_data(
            &self,
            _order_id: crate::domain::model::order::OrderId,
            _train_schedule_id: crate::domain::model::train_schedule::TrainScheduleId,
            _tz_offset_hour: i32,
        ) -> Result<crate::domain::repository::order::TrainOrderRelatedData, RepositoryError>
        {
            Err(RepositoryError::Db(anyhow::anyhow!("not implemented")))
        }
        async fn get_hotel_order_related_data(
            &self,
            _order_id: crate::domain::model::order::OrderId,
        ) -> Result<crate::domain::repository::order::HotelOrderRelatedData, RepositoryError>
        {
            Err(RepositoryError::Db(anyhow::anyhow!("not implemented")))
        }
        async fn get_dish_order_related_data(
            &self,
            _order_id: crate::domain::model::order::OrderId,
            _tz_offset_hour: i32,
        ) -> Result<crate::domain::repository::order::DishOrderRelatedData, RepositoryError>
        {
            Err(RepositoryError::Db(anyhow::anyhow!("not implemented")))
        }
        async fn get_takeaway_order_related_data(
            &self,
            _order_id: crate::domain::model::order::OrderId,
            _train_order_id: crate::domain::model::order::OrderId,
            _tz_offset_hour: i32,
        ) -> Result<crate::domain::repository::order::TakeawayOrderRelatedData, RepositoryError>
        {
            Err(RepositoryError::Db(anyhow::anyhow!("not implemented")))
        }
        async fn verify_train_order(
            &self,
            _user_id: crate::domain::model::user::UserId,
            _train_number: String,
            _origin_departure_date: chrono::NaiveDate,
            _origin_departure_time_second: i32,
        ) -> Result<bool, RepositoryError> {
            Ok(false)
        }
    }

    struct MockTransactionService {
        pub refund_ok: bool,
    }
    #[async_trait]
    impl TransactionService for MockTransactionService {
        async fn recharge(
            &self,
            _user_id: crate::domain::model::user::UserId,
            _amount: crate::domain::model::transaction::TransactionAmountAbs,
        ) -> Result<Uuid, TransactionServiceError> {
            Err(TransactionServiceError::InvalidUser(1.into()))
        }
        async fn get_balance(
            &self,
            _user_id: crate::domain::model::user::UserId,
        ) -> Result<Decimal, TransactionServiceError> {
            Ok(Decimal::ZERO)
        }
        async fn new_transaction(
            &self,
            _user_id: crate::domain::model::user::UserId,
            _orders: Vec<Box<dyn Order>>,
            _atomic: bool,
        ) -> Result<Uuid, TransactionServiceError> {
            Ok(Uuid::new_v4())
        }
        async fn pay_transaction(
            &self,
            _transaction_id: Uuid,
        ) -> Result<(), TransactionServiceError> {
            Ok(())
        }
        async fn refund_transaction(
            &self,
            _transaction_id: Uuid,
            _to_refund_orders: &[Box<dyn Order>],
        ) -> Result<Uuid, TransactionServiceError> {
            if self.refund_ok {
                Ok(Uuid::new_v4())
            } else {
                Err(TransactionServiceError::InvalidTransactionId(Uuid::new_v4()))
            }
        }
        async fn convert_transaction_to_dto(
            &self,
            _transaction: crate::domain::model::transaction::Transaction,
        ) -> Result<
            crate::domain::service::order::order_dto::TransactionDataDto,
            TransactionServiceError,
        > {
            Err(TransactionServiceError::InvalidTransactionId(Uuid::new_v4()))
        }
    }

    struct MockSessionManagerService {
        pub user: Option<crate::domain::model::user::UserId>,
    }
    #[async_trait]
    impl SessionManagerService for MockSessionManagerService {
        async fn create_session(
            &self,
            _user_id: crate::domain::model::user::UserId,
        ) -> Result<Session, RepositoryError> {
            Err(RepositoryError::Db(anyhow::anyhow!("not implemented")))
        }
        async fn delete_session(&self, _session: Session) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn get_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<Session>, RepositoryError> {
            Ok(None)
        }
        async fn get_user_id_by_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<crate::domain::model::user::UserId>, RepositoryError> {
            Ok(self.user)
        }
        async fn verify_session_id(&self, _session_id_str: &str) -> Result<bool, RepositoryError> {
            Ok(true)
        }
    }

    struct MockPersonalInfoRepository {
        pub list: Vec<PersonalInfo>,
    }
    #[async_trait]
    impl PersonalInfoRepository for MockPersonalInfoRepository {
        async fn find_by_user_id(
            &self,
            _user_id: crate::domain::model::user::UserId,
        ) -> Result<Vec<PersonalInfo>, RepositoryError> {
            Ok(self.list.clone())
        }
        async fn find_by_user_id_and_identity_card(
            &self,
            _user_id: crate::domain::model::user::UserId,
            _identity_card_id: crate::domain::model::user::IdentityCardId,
        ) -> Result<Option<PersonalInfo>, RepositoryError> {
            Ok(None)
        }
    }

    #[async_trait]
    impl crate::domain::Repository<PersonalInfo> for MockPersonalInfoRepository {
        async fn find(&self, _id: PersonalInfoId) -> Result<Option<PersonalInfo>, RepositoryError> {
            Ok(None)
        }
        async fn remove(&self, _aggregate: PersonalInfo) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(
            &self,
            _aggregate: &mut PersonalInfo,
        ) -> Result<PersonalInfoId, RepositoryError> {
            Ok(PersonalInfoId::from(1u64))
        }
    }

    // --------- 帮助函数：构造一个可用的 Hotel ---------
    fn build_hotel() -> Hotel {
        let city = City::new(
            Some(CityId::from(1u64)),
            CityName::from("北京".to_string()),
            ProvinceName::from("北京".to_string()),
        );
        let station = Station::new(
            Some(StationId::from(1u64)),
            "北京南".into(),
            CityId::from(1u64),
        );
        let mut hotel = Hotel::new("H1".into(), city, station, "Addr".into(), "Info".into());
        hotel.set_id(HotelId::from(1u64));
        let rt = HotelRoomType::new(
            Some(HotelRoomTypeId::from(1u64)),
            Some(HotelId::from(1u64)),
            "总统套房".into(),
            2,
            Decimal::from(100),
        );
        hotel.add_room_type(rt);
        hotel
    }

    fn build_personal_info(
        user_id: crate::domain::model::user::UserId,
        uuid: Uuid,
    ) -> PersonalInfo {
        let name = RealName::try_from("Tomori Takamatsu".to_string()).unwrap();
        let id = IdentityCardId::try_from("11010519491231002X".to_string()).unwrap();
        PersonalInfo::new(
            Some(PersonalInfoId::from(1u64)),
            uuid,
            name,
            id,
            None,
            user_id,
        )
    }

    fn build_order_for_refund() -> crate::domain::model::order::HotelOrder {
        let now: DateTimeWithTimeZone = chrono::Local::now().into();
        let time = OrderTimeInfo::new(now, now, now);
        let base = BaseOrder::new(
            None,
            Uuid::new_v4(),
            OrderStatus::Unpaid,
            time,
            Decimal::from(100),
            Decimal::from(1),
            PaymentInfo::new(None, None),
            PersonalInfoId::from(1u64),
        );
        crate::domain::model::order::HotelOrder::new(
            base,
            HotelId::from(1u64),
            HotelRoomTypeId::from(1u64),
            HotelDateRange::new(
                chrono::NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2025, 9, 2).unwrap(),
            )
            .unwrap(),
        )
    }

    // --------- 测试：process_hotel_orders 成功 ---------
    #[tokio::test]
    async fn test_process_hotel_orders_success_impl() {
        let hotel = build_hotel();
        let hotel_uuid = hotel.uuid();
        let hotel_repo = Arc::new(MockHotelRepository { hotel: Some(hotel) });
        let booking = Arc::new(MockHotelBookingService { ok: true });
        let order_repo = Arc::new(MockOrderRepository { hotel_order: None });
        let txn = Arc::new(MockTransactionService { refund_ok: true });
        let session = Arc::new(MockSessionManagerService {
            user: Some(1.into()),
        });

        let personal_info_uuid = Uuid::new_v4();

        let pir = Arc::new(MockPersonalInfoRepository {
            list: vec![build_personal_info(1.into(), personal_info_uuid)],
        });

        let service =
            HotelOrderServiceImpl::new(hotel_repo, booking, order_repo, txn, session, pir);

        let req = HotelOrderRequestDTO {
            hotel_id: hotel_uuid.to_string(),
            room_type: "总统套房".into(),
            begin_date: Some("2025-09-01".into()),
            end_date: Some("2025-09-02".into()),
            personal_id: personal_info_uuid.to_string(),
            amount: 2,
        };
        let res = service
            .process_hotel_orders(Uuid::new_v4().to_string(), vec![req])
            .await
            .unwrap();
        assert_eq!(res.status, "unpaid");
        assert!(res.amount > 0.0);
    }

    // --------- 反例：空订单 ---------
    #[tokio::test]
    async fn test_process_hotel_orders_empty() {
        let hotel_repo = Arc::new(MockHotelRepository { hotel: None });
        let booking = Arc::new(MockHotelBookingService { ok: true });
        let order_repo = Arc::new(MockOrderRepository { hotel_order: None });
        let txn = Arc::new(MockTransactionService { refund_ok: true });
        let session = Arc::new(MockSessionManagerService {
            user: Some(1.into()),
        });
        let pir = Arc::new(MockPersonalInfoRepository { list: vec![] });
        let service =
            HotelOrderServiceImpl::new(hotel_repo, booking, order_repo, txn, session, pir);

        let err = service
            .process_hotel_orders(Uuid::new_v4().to_string(), vec![])
            .await
            .err()
            .unwrap();
        assert!(err.error_message().contains("Empty order list"));
    }

    // --------- 反例：会话无效 ---------
    #[tokio::test]
    async fn test_process_hotel_orders_invalid_session() {
        let hotel_repo = Arc::new(MockHotelRepository { hotel: None });
        let booking = Arc::new(MockHotelBookingService { ok: true });
        let order_repo = Arc::new(MockOrderRepository { hotel_order: None });
        let txn = Arc::new(MockTransactionService { refund_ok: true });
        let session = Arc::new(MockSessionManagerService { user: None });
        let pir = Arc::new(MockPersonalInfoRepository { list: vec![] });
        let service =
            HotelOrderServiceImpl::new(hotel_repo, booking, order_repo, txn, session, pir);

        let req = HotelOrderRequestDTO {
            hotel_id: Uuid::new_v4().to_string(),
            room_type: "总统套房".into(),
            begin_date: Some("2025-09-01".into()),
            end_date: Some("2025-09-02".into()),
            personal_id: Uuid::new_v4().to_string(),
            amount: 1,
        };
        let err = service
            .process_hotel_orders(Uuid::new_v4().to_string(), vec![req])
            .await
            .err()
            .unwrap();
        assert_eq!(err.error_code(), 403);
    }

    // --------- 测试：process_order_message 成功与失败退款 ---------
    #[tokio::test]
    async fn test_process_order_message_success() {
        let service = HotelOrderServiceImpl::new(
            Arc::new(MockHotelRepository { hotel: None }),
            Arc::new(MockHotelBookingService { ok: true }),
            Arc::new(MockOrderRepository { hotel_order: None }),
            Arc::new(MockTransactionService { refund_ok: true }),
            Arc::new(MockSessionManagerService {
                user: Some(1.into()),
            }),
            Arc::new(MockPersonalInfoRepository { list: vec![] }),
        );

        let res = service
            .process_order_message(Uuid::new_v4(), vec![Uuid::new_v4()], true)
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_process_order_message_fail_and_refund() {
        let service = HotelOrderServiceImpl::new(
            Arc::new(MockHotelRepository { hotel: None }),
            Arc::new(MockHotelBookingService { ok: false }),
            Arc::new(MockOrderRepository {
                hotel_order: Some(build_order_for_refund()),
            }),
            Arc::new(MockTransactionService { refund_ok: true }),
            Arc::new(MockSessionManagerService {
                user: Some(1.into()),
            }),
            Arc::new(MockPersonalInfoRepository { list: vec![] }),
        );

        let res = service
            .process_order_message(Uuid::new_v4(), vec![Uuid::new_v4()], true)
            .await;
        assert!(res.is_err());
    }
}
