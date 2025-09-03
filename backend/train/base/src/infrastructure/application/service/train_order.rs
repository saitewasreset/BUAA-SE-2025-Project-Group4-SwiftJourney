use crate::application::service::train_order::CreateTrainOrderDTO;
use crate::application::service::train_order::OrderPackDTO;
use crate::application::service::train_order::TrainOrderService;
use crate::application::service::train_order::TrainOrderServiceError;
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::train_booking::TrainBookingService;
use crate::domain::service::train_schedule::TrainScheduleService;
use crate::domain::service::train_type::TrainTypeConfigurationService;
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::Timelike;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use shared::application_error::ApplicationError;
use shared::application_error::GeneralError;
use shared::domain::Identifiable;
use shared::domain::ServiceError;
use shared::domain::model::order::{
    BaseOrder, Order, OrderStatus, OrderTimeInfo, PaymentInfo, TrainOrder,
};
use shared::domain::model::personal_info::{PersonalInfoId, PreferredSeatLocation};
use shared::domain::model::session::SessionId;
use shared::domain::model::station::StationId;
use shared::domain::model::train::SeatTypeName;
use shared::domain::model::train_schedule::StationRange;
use shared::domain::model::user::UserId;
use shared::internal::order::command::{
    NewTransactionCommand, OrderByUuidQuery, RefundTransactionCommand,
};
use shared::internal::order::dto::TransactionInfoDTO;
use shared::internal::user::command::SessionQuery;
use shared::ports::geo::GeoPort;
use shared::ports::order::OrderPort;
use shared::ports::user::UserPort;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct TrainOrderServiceImpl<TSR, TBS, TR, RR, GP, OP, UP, TSS, TTCS>
where
    TSR: TrainScheduleRepository,
    TBS: TrainBookingService,
    TR: TrainRepository,
    RR: RouteRepository,
    GP: GeoPort,
    OP: OrderPort,
    UP: UserPort,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
{
    train_schedule_repository: Arc<TSR>,
    train_booking_service: Arc<TBS>,
    train_repository: Arc<TR>,
    route_repository: Arc<RR>,
    train_schedule_service: Arc<TSS>,
    train_type_configuration_service: Arc<TTCS>,
    geo_port: Arc<GP>,
    order_port: Arc<OP>,
    user_port: Arc<UP>,
}

impl<TSR, TBS, TR, RR, GP, OP, UP, TSS, TTCS>
    TrainOrderServiceImpl<TSR, TBS, TR, RR, GP, OP, UP, TSS, TTCS>
where
    TSR: TrainScheduleRepository,
    TBS: TrainBookingService,
    TR: TrainRepository,
    RR: RouteRepository,
    GP: GeoPort,
    OP: OrderPort,
    UP: UserPort,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        train_schedule_repository: Arc<TSR>,
        train_booking_service: Arc<TBS>,
        train_repository: Arc<TR>,
        route_repository: Arc<RR>,
        train_schedule_service: Arc<TSS>,
        train_type_configuration_service: Arc<TTCS>,
        geo_port: Arc<GP>,
        order_port: Arc<OP>,
        user_port: Arc<UP>,
    ) -> Self {
        Self {
            train_schedule_repository,
            train_booking_service,
            train_repository,
            route_repository,
            train_schedule_service,
            train_type_configuration_service,
            geo_port,
            order_port,
            user_port,
        }
    }

    #[instrument(skip(self))]
    async fn validate_and_create_train_order(
        &self,
        dto: &CreateTrainOrderDTO,
        user_id: UserId,
    ) -> Result<Box<dyn Order>, Box<dyn ApplicationError>> {
        let train_number = self
            .train_type_configuration_service
            .verify_train_number(dto.train_number.clone().into())
            .await
            .map_err(|e| {
                warn!("{:?}", e);

                GeneralError::NotFound(format!("Invalid train number: {}", dto.train_number))
            })?;

        let train = self
            .train_repository
            .find_by_train_number(train_number)
            .await
            .map_err(|e| {
                error!("Database error when finding train: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let train_id = train.get_id().ok_or_else(|| {
            error!("Failed to get train ID from train entity");
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        let schedules_result = self
            .train_schedule_repository
            .find_by_train_id(train_id)
            .await
            .map_err(|e| {
                error!("Database error when finding train schedules: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let origin_departure_time =
            DateTimeWithTimeZone::parse_from_rfc3339(&dto.origin_departure_time).map_err(|e| {
                GeneralError::BadRequest(format!("Invalid origin departure time format: {}", e))
            })?;

        let train_schedule = schedules_result
            .iter()
            .find(|schedule| {
                let date = origin_departure_time.date_naive();

                let origin_departure_seconds =
                    origin_departure_time.time().num_seconds_from_midnight() as i32;

                schedule.date() == date
                    && schedule.origin_departure_time() == origin_departure_seconds
            })
            .cloned()
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let route_id = train_schedule.route_id();

        let route = self
            .route_repository
            .find(route_id)
            .await
            .map_err(|e| {
                error!("Database error when finding route: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or_else(|| {
                error!(
                    "Data inconsistency: Route {} referenced by train schedule not found",
                    route_id
                );
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let stations = route.stops();

        let db_station_list = self
            .geo_port
            .db_get_stations()
            .await
            .inspect_err(|e| error!("Failed to load db stations: {:?}", e))
            .map_err(|e| GeneralError::InternalServerError)?;

        let station_map = db_station_list
            .into_iter()
            .map(|x| (StationId::from(x.id as u64), x))
            .collect::<HashMap<_, _>>();

        // 验证出发站和到达站
        let mut departure_exists = false;
        let mut arrival_exists = false;
        let mut departure_station_id = None;
        let mut arrival_station_id = None;

        for stop in stations {
            if let Some(station) = station_map.get(&stop.station_id()) {
                let station_name = station.name.clone();

                if station_name == dto.departure_station {
                    departure_exists = true;
                    departure_station_id = Some(StationId::from(station.id as u64));
                }
                if station_name == dto.arrival_station {
                    arrival_exists = true;
                    arrival_station_id = Some(StationId::from(station.id as u64));
                }

                if departure_exists && arrival_exists {
                    break;
                }
            }
        }

        if !departure_exists || !arrival_exists {
            return Err(Box::new(TrainOrderServiceError::InvalidStationId));
        }

        let train_details = self
            .train_repository
            .find(train_id)
            .await
            .map_err(|e| {
                error!("Database error when finding train details: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or_else(|| {
                error!(
                    "Data inconsistency: Train details for ID {} not found despite train existing",
                    train_id
                );
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        // 警告：若要修改下一行代码，需要验证创建order_seat_type_name时的SAFETY要求是否仍然满足
        let seat_type_exists = train_details
            .seats()
            .iter()
            .any(|(key, _)| key == &dto.seat_type);

        if !seat_type_exists {
            return Err(GeneralError::NotFound(format!(
                "no seat type {} found for train number {}",
                dto.seat_type, dto.train_number
            ))
            .into());
        }

        // SAFETY: dto.seat_type 已经在上面验证过存在
        let order_seat_type_name = SeatTypeName::from_unchecked(dto.seat_type.clone());

        // === 创建订单 ===

        let order_uuid = Uuid::new_v4();

        // SAFETY: 这里的 departure_station_id 和 arrival_station_id 已经在上面验证过了
        let station_range = StationRange::from_unchecked(
            departure_station_id.unwrap(),
            arrival_station_id.unwrap(),
        );

        let now = sea_orm::prelude::DateTimeWithTimeZone::from(chrono::Local::now());

        let train_schedule_id = train_schedule.get_id().ok_or_else(|| {
            error!("Failed to get train schedule ID");
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        let departure_arrival_time = self
            .train_schedule_service
            .get_station_arrival_time(train_schedule_id, departure_station_id.unwrap())
            .await
            .map_err(|e| {
                error!("Failed to get station arrival time: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let arrival_arrival_time = self
            .train_schedule_service
            .get_station_arrival_time(train_schedule_id, arrival_station_id.unwrap())
            .await
            .map_err(|e| {
                error!("Failed to get station arrival time: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let order_time_info = OrderTimeInfo::new(now, departure_arrival_time, arrival_arrival_time);

        let payment_info = PaymentInfo::new(
            None, // 还未支付
            None, // 还未退款
        );

        let personal_uuid = match Uuid::parse_str(&dto.personal_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(Box::new(TrainOrderServiceError::InvalidPassengerId)),
        };

        let personal_infos = self
            .user_port
            .db_get_personal_info()
            .await
            .inspect_err(|e| error!("Failed to get db personal info: {:?}", e))
            .map_err(|e| GeneralError::InternalServerError)?;

        let personal_info = personal_infos
            .into_iter()
            .find(|info| info.uuid == personal_uuid)
            .ok_or(TrainOrderServiceError::InvalidPassengerId)?;

        let personal_info_id = PersonalInfoId::from(personal_info.id as u64);

        let seat_type = train_details
            .seats()
            .get(&dto.seat_type)
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let unit_price = seat_type.unit_price();

        let mut departure_index = None;
        let mut arrival_index = None;

        for (index, stop) in route.stops().iter().enumerate() {
            if let Some(station) = station_map.get(&stop.station_id()) {
                let station_name = station.name.clone();

                if station_name == dto.departure_station {
                    departure_index = Some(index);
                }
                if station_name == dto.arrival_station {
                    arrival_index = Some(index);
                }

                if departure_index.is_some() && arrival_index.is_some() {
                    break;
                }
            }
        }

        let stations_count = match (departure_index, arrival_index) {
            (Some(d), Some(a)) => {
                if a > d {
                    (a - d) as i64
                } else {
                    return Err(Box::new(TrainOrderServiceError::InvalidStationId));
                }
            }
            _ => return Err(Box::new(TrainOrderServiceError::InvalidStationId)),
        };

        let total_price = unit_price * Decimal::from(stations_count);

        let base_order = BaseOrder::new(
            None,
            order_uuid,
            OrderStatus::Unpaid,
            order_time_info,
            total_price,
            Decimal::from(1), // 一笔订单应该对应一张票
            payment_info,
            personal_info_id,
        );

        let train_order = TrainOrder::new(
            base_order,
            train_schedule
                .get_id()
                .expect("The train schedule is invalid"),
            None,
            order_seat_type_name,
            personal_info
                .preferred_seat_location
                .map(|x| PreferredSeatLocation::try_from(x.chars().next().unwrap()).unwrap()),
            station_range,
        );

        Ok(Box::new(train_order))
    }

    // 处理订单消息（模拟消息队列消费者处理）
    pub async fn process_order_message(
        &self,
        transaction_id: Uuid,
        order_uuids: Vec<Uuid>,
        atomic: bool,
    ) -> Result<(), TrainOrderServiceError> {
        info!("Processing orders for transaction: {}", transaction_id);

        // 调用booking_group处理订单
        let result = self
            .train_booking_service
            .booking_group(order_uuids.clone(), atomic)
            .await;

        match result {
            Ok(_) => {
                info!(
                    "Successfully processed orders for transaction: {}",
                    transaction_id
                );
                Ok(())
            }
            Err(err) => {
                error!(
                    "Failed to process orders for transaction {}: {:?}",
                    transaction_id, err
                );

                // 自动触发退款流程
                info!(
                    "Initiating automatic refund for failed transaction: {}",
                    transaction_id
                );

                let mut to_refund_orders: Vec<Box<dyn Order>> = Vec::new();

                for order_uuid in order_uuids {
                    match self
                        .order_port
                        .get_order_by_uuid(OrderByUuidQuery { order_uuid })
                        .await
                    {
                        Ok(Some(order)) => {
                            info!("Found order {} for refund", order_uuid);
                            to_refund_orders.push(order.into());
                        }
                        Ok(None) => {
                            error!(
                                "Data inconsistency: Order {} not found for refund despite being created earlier",
                                order_uuid
                            );
                            return Err(TrainOrderServiceError::InfrastructureError(
                                ServiceError::RelatedServiceError(anyhow!(
                                    "Order {} not found for refund",
                                    order_uuid
                                )),
                            ));
                        }
                        Err(err) => {
                            error!(
                                "Database error finding order {} for refund: {:?}",
                                order_uuid, err
                            );
                            return Err(TrainOrderServiceError::InfrastructureError(
                                ServiceError::RepositoryError(
                                    anyhow!("Error finding order: {:?}", err).into(),
                                ),
                            ));
                        }
                    }
                }

                if let Err(refund_err) = self
                    .order_port
                    .refund_transaction(RefundTransactionCommand {
                        transaction_id,
                        to_refund_orders: to_refund_orders
                            .into_iter()
                            .map(|x| x.as_ref().into())
                            .collect(),
                    })
                    .await
                {
                    error!(
                        "Failed to process automatic refund for transaction {}: {:?}",
                        transaction_id, refund_err
                    );
                } else {
                    info!(
                        "Automatic refund successfully initiated for transaction: {}",
                        transaction_id
                    );
                }

                Err(TrainOrderServiceError::InvalidTrainNumber)
            }
        }
    }
}

#[async_trait]
impl<TSR, TBS, TR, RR, GP, OP, UP, TSS, TTCS> TrainOrderService
    for TrainOrderServiceImpl<TSR, TBS, TR, RR, GP, OP, UP, TSS, TTCS>
where
    TSR: TrainScheduleRepository,
    TBS: TrainBookingService,
    TR: TrainRepository,
    RR: RouteRepository,
    GP: GeoPort,
    OP: OrderPort,
    UP: UserPort,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
{
    #[instrument(skip_all)]
    async fn process_train_order_packs(
        &self,
        session_id: String,
        order_packs: Vec<OrderPackDTO>,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        let user_id = UserId::from(
            self.user_port
                .get_session(SessionQuery { session_id })
                .await
                .map_err(|e| {
                    error!("Failed to get user ID by session: {:?}", e);
                    TrainOrderServiceError::InvalidSessionId
                })?
                .ok_or(TrainOrderServiceError::InvalidSessionId)?
                .user_id,
        );

        let mut all_train_orders: Vec<Box<dyn Order>> = Vec::new();
        let mut all_order_uuids: Vec<Uuid> = Vec::new();
        let mut all_atomic = true;
        let mut total_amount: f64 = 0.0;

        for pack in order_packs {
            all_atomic &= pack.atomic;

            for order_request in pack.order_list {
                let dto = CreateTrainOrderDTO {
                    train_number: order_request.train_number.clone(),
                    origin_departure_time: order_request.origin_departure_time.clone(),
                    departure_station: order_request.departure_station.clone(),
                    arrival_station: order_request.arrival_station.clone(),
                    personal_id: order_request.personal_id.clone(),
                    seat_type: order_request.seat_type.clone(),
                };

                let train_order = self.validate_and_create_train_order(&dto, user_id).await?;

                total_amount += (train_order.unit_price() * train_order.amount())
                    .to_f64()
                    .expect("Failed to convert amount to f64");

                all_order_uuids.push(train_order.uuid());
                all_train_orders.push(train_order);
            }
        }

        let transaction_id = self
            .order_port
            .new_transaction(NewTransactionCommand {
                user_id: user_id.into(),
                orders: all_train_orders
                    .into_iter()
                    .map(|x| x.as_ref().into())
                    .collect(),
                atomic: all_atomic,
            })
            .await
            .map_err(|e| {
                TrainOrderServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    e.into(),
                ))
            })?;

        Ok(TransactionInfoDTO {
            transaction_id,
            amount: total_amount,
            status: "unpaid".to_string(),
        })
    }
}
