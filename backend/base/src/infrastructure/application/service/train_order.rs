use crate::application::ApplicationError;
use crate::application::GeneralError;
use crate::application::service::train_order::CreateTrainOrderDTO;
use crate::application::service::train_order::OrderPackDTO;
use crate::application::service::train_order::TrainOrderService;
use crate::application::service::train_order::TrainOrderServiceError;
use crate::application::service::transaction::TransactionInfoDTO;
use crate::domain::Identifiable;
use crate::domain::model::order::{
    BaseOrder, Order, OrderStatus, OrderTimeInfo, PaymentInfo, TrainOrder,
};
use crate::domain::model::session::SessionId;
use crate::domain::model::train::{SeatTypeName, TrainNumber};
use crate::domain::model::train_schedule::StationRange;
use crate::domain::model::user::UserId;
use crate::domain::repository::order::OrderRepository;
use crate::domain::repository::personal_info::PersonalInfoRepository;
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::train_booking::TrainBookingService;
use crate::domain::service::train_schedule::TrainScheduleService;
use crate::domain::service::transaction::TransactionService;
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::Timelike;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

#[derive(Clone)]
pub struct TrainOrderServiceImpl<TSR, TBS, TR, RR, SR, OR, TS, SMS, PIR, TSS>
where
    TSR: TrainScheduleRepository,
    TBS: TrainBookingService,
    TR: TrainRepository,
    RR: RouteRepository,
    SR: StationRepository,
    OR: OrderRepository,
    TS: TransactionService,
    SMS: SessionManagerService,
    PIR: PersonalInfoRepository,
    TSS: TrainScheduleService,
{
    train_schedule_repository: Arc<TSR>,
    train_booking_service: Arc<TBS>,
    train_repository: Arc<TR>,
    route_repository: Arc<RR>,
    station_repository: Arc<SR>,
    order_repository: Arc<OR>,
    transaction_service: Arc<TS>,
    session_manager_service: Arc<SMS>,
    personal_info_repository: Arc<PIR>,
    train_schedule_service: Arc<TSS>,
}

impl<TSR, TBS, TR, RR, SR, OR, TS, SMS, PIR, TSS>
    TrainOrderServiceImpl<TSR, TBS, TR, RR, SR, OR, TS, SMS, PIR, TSS>
where
    TSR: TrainScheduleRepository,
    TBS: TrainBookingService,
    TR: TrainRepository,
    RR: RouteRepository,
    SR: StationRepository,
    OR: OrderRepository,
    TS: TransactionService,
    SMS: SessionManagerService,
    PIR: PersonalInfoRepository,
    TSS: TrainScheduleService,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        train_schedule_repository: Arc<TSR>,
        train_booking_service: Arc<TBS>,
        train_repository: Arc<TR>,
        route_repository: Arc<RR>,
        station_repository: Arc<SR>,
        order_repository: Arc<OR>,
        transaction_service: Arc<TS>,
        session_manager_service: Arc<SMS>,
        personal_info_repository: Arc<PIR>,
        train_schedule_service: Arc<TSS>,
    ) -> Self {
        Self {
            train_schedule_repository,
            train_booking_service,
            train_repository,
            route_repository,
            station_repository,
            order_repository,
            transaction_service,
            session_manager_service,
            personal_info_repository,
            train_schedule_service,
        }
    }

    async fn validate_and_create_train_order(
        &self,
        dto: &CreateTrainOrderDTO,
        user_id: UserId,
    ) -> Result<Box<dyn Order>, Box<dyn ApplicationError>> {
        // == 验证订单 ==
        // SAFETY: 正确性将在find_by_train_number中检查
        let train_number = TrainNumber::from_unchecked(dto.train_number.clone());

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

        let station_ids: Vec<_> = stations.iter().map(|stop| stop.station_id()).collect();

        let mut station_map = std::collections::HashMap::new();
        for station_id in station_ids {
            if let Ok(Some(station)) = self.station_repository.find(station_id).await {
                station_map.insert(station_id, station);
            }
        }

        // 验证出发站和到达站
        let mut departure_exists = false;
        let mut arrival_exists = false;
        let mut departure_station_id = None;
        let mut arrival_station_id = None;

        for stop in stations {
            if let Some(station) = station_map.get(&stop.station_id()) {
                let station_name = station.name().to_string();

                if station_name == dto.departure_station {
                    departure_exists = true;
                    departure_station_id = station.get_id();
                }
                if station_name == dto.arrival_station {
                    arrival_exists = true;
                    arrival_station_id = station.get_id();
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
            return Err(Box::new(TrainOrderServiceError::InvalidTrainNumber));
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
            .ok_or(TrainOrderServiceError::InvalidPassengerId)?;

        let personal_info_id = personal_info
            .get_id()
            .ok_or(TrainOrderServiceError::InvalidPassengerId)?;

        let seat_type = train_details
            .seats()
            .get(&dto.seat_type)
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let unit_price = seat_type.unit_price();

        let mut departure_index = None;
        let mut arrival_index = None;

        for (index, stop) in route.stops().iter().enumerate() {
            if let Some(station) = station_map.get(&stop.station_id()) {
                let station_name = station.name().to_string();

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
            personal_info.preferred_seat_location(),
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
                        .order_repository
                        .find_train_order_by_uuid(order_uuid)
                        .await
                    {
                        Ok(Some(order)) => {
                            info!("Found order {} for refund", order_uuid);
                            to_refund_orders.push(Box::new(order));
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
                    .transaction_service
                    .refund_transaction(transaction_id, &to_refund_orders)
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
impl<TSR, TBS, TR, RR, SR, OR, TS, SMS, PIR, TSS> TrainOrderService
    for TrainOrderServiceImpl<TSR, TBS, TR, RR, SR, OR, TS, SMS, PIR, TSS>
where
    TSR: TrainScheduleRepository + Send + Sync + 'static,
    TBS: TrainBookingService + Send + Sync + 'static,
    TR: TrainRepository + Send + Sync + 'static,
    RR: RouteRepository + Send + Sync + 'static,
    SR: StationRepository + Send + Sync + 'static,
    OR: OrderRepository + Send + Sync + 'static,
    TS: TransactionService + Send + Sync + 'static,
    SMS: SessionManagerService + Send + Sync + 'static,
    PIR: PersonalInfoRepository + Send + Sync + 'static,
    TSS: TrainScheduleService + Send + Sync + 'static,
{
    #[instrument(skip_all)]
    async fn process_train_order_packs(
        &self,
        session_id: String,
        order_packs: Vec<OrderPackDTO>,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        let user_id = self
            .session_manager_service
            .get_user_id_by_session(
                SessionId::try_from(session_id.as_str())
                    .map_err(|_| TrainOrderServiceError::InvalidSessionId)?,
            )
            .await
            .map_err(|e| {
                error!("Failed to get user ID by session: {:?}", e);
                TrainOrderServiceError::InvalidSessionId
            })?
            .ok_or(TrainOrderServiceError::InvalidSessionId)?;

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
            .transaction_service
            .new_transaction(user_id, all_train_orders, all_atomic)
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
