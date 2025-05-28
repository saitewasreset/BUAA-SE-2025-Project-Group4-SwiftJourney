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

        let room_type = hotel
            .room_type_list()
            .iter()
            .find(|rt| rt.type_name() == &dto.room_type)
            .ok_or(Box::new(GeneralError::NotFound) as Box<dyn ApplicationError>)?;

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
            .ok_or(Box::new(GeneralError::NotFound) as Box<dyn ApplicationError>)?;

        let personal_info_id = personal_info
            .get_id()
            .ok_or(Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>)?;

        let days = date_range
            .end_date()
            .signed_duration_since(date_range.begin_date())
            .num_days() as u32;
        let room_price = room_price_per_day * Decimal::from(days);
        let amount = Decimal::from(dto.amount);

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
