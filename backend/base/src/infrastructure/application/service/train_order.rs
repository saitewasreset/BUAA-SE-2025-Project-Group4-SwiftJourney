use crate::application::service::train_order::CreateTrainOrderDTO;
use crate::application::service::train_order::OrderPackDTO;
use crate::application::service::train_order::TrainOrderService;
use crate::application::service::train_order::TrainOrderServiceError;
use crate::application::service::transaction::TransactionInfoDTO;
use crate::application::ApplicationError;
use crate::domain::Identifiable;
use crate::domain::model::order::{
    BaseOrder, Order, OrderStatus, OrderTimeInfo, PaymentInfo, TrainOrder,
};
use crate::domain::model::session::SessionId;
use crate::domain::model::train::TrainNumber;
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
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;
use tracing::{error, info};
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
    ) -> Result<(Uuid, Box<dyn Order>), TrainOrderServiceError> {
        // == 验证订单 ==
        // SAFETY: 正确性将在find_by_train_number中检查
        let train_number = TrainNumber::from_unchecked(dto.train_number.clone());

        let train = self
            .train_repository
            .find_by_train_number(train_number)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidTrainNumber)?;

        let train_id = train
            .get_id()
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let schedules_result = self
            .train_schedule_repository
            .find_by_train_id(train_id)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidTrainNumber)?;

        let train_schedule = schedules_result
            .iter()
            .find(|schedule| {
                schedule.origin_departure_time().to_string() == dto.origin_departure_time
            })
            .cloned()
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let route_id = train_schedule.route_id();

        let route = self
            .route_repository
            .find(route_id)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidStationId)?
            .ok_or(TrainOrderServiceError::InvalidStationId)?;

        let stations = route.stops();

        let mut departure_exists = false;
        let mut arrival_exists = false;
        let mut departure_station_id = None;
        let mut arrival_station_id = None;

        for stop in stations {
            if let Ok(Some(station)) = self.station_repository.find(stop.station_id()).await {
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
            return Err(TrainOrderServiceError::InvalidStationId);
        }

        let train_details = self
            .train_repository
            .find(train_id)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidTrainNumber)?
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let seat_type_exists = train_details
            .seats()
            .iter()
            .any(|(key, _)| key == &dto.seat_type);

        if !seat_type_exists {
            return Err(TrainOrderServiceError::InvalidTrainNumber);
        }

        // === 创建订单 ===

        let order_uuid = Uuid::new_v4();

        // SAFETY: 这里的 departure_station_id 和 arrival_station_id 已经在上面验证过了
        let station_range = StationRange::from_unchecked(
            departure_station_id.unwrap(),
            arrival_station_id.unwrap(),
        );

        let now = sea_orm::prelude::DateTimeWithTimeZone::from(chrono::Utc::now());

        let train_schedule_id = train_schedule
            .get_id()
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let departure_arrival_time = self
            .train_schedule_service
            .get_station_arrival_time(train_schedule_id, departure_station_id.unwrap())
            .await
            .map_err(|_| TrainOrderServiceError::InvalidStationId)?;

        let arrival_arrival_time = self
            .train_schedule_service
            .get_station_arrival_time(train_schedule_id, arrival_station_id.unwrap())
            .await
            .map_err(|_| TrainOrderServiceError::InvalidStationId)?;

        let order_time_info = OrderTimeInfo::new(now, departure_arrival_time, arrival_arrival_time);

        let payment_info = PaymentInfo::new(
            None, // 还未支付
            None, // 还未退款
        );

        let personal_uuid = match Uuid::parse_str(&dto.personal_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(TrainOrderServiceError::InvalidPassengerId),
        };

        let personal_infos = self
            .personal_info_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidPassengerId)?;

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
            if let Ok(Some(station)) = self.station_repository.find(stop.station_id()).await {
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
                    return Err(TrainOrderServiceError::InvalidStationId);
                }
            }
            _ => return Err(TrainOrderServiceError::InvalidStationId),
        };

        // let total_price = unit_price * Decimal::from(stations_count);

        let base_order = BaseOrder::new(
            None,
            order_uuid,
            OrderStatus::Unpaid,
            order_time_info,
            unit_price,
            stations_count.into(),
            payment_info,
            personal_info_id,
        );

        let train_order = TrainOrder::new(
            base_order,
            train_schedule
                .get_id()
                .expect("The train schedule is invalid"),
            None,
            personal_info.preferred_seat_location(),
            station_range,
        );

        Ok((order_uuid, Box::new(train_order)))
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
                            error!("Order {} not found for refund", order_uuid);
                            return Err(TrainOrderServiceError::InvalidTrainNumber);
                        }
                        Err(err) => {
                            error!("Error finding order {} for refund: {:?}", order_uuid, err);
                            return Err(TrainOrderServiceError::InvalidTrainNumber);
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

                let (order_uuid, train_order) =
                    self.validate_and_create_train_order(&dto, user_id).await?;

                total_amount += (train_order.unit_price() * train_order.amount())
                    .to_f64()
                    .expect("Failed to convert amount to f64");

                all_order_uuids.push(order_uuid);
                all_train_orders.push(train_order);
            }
        }

        let transaction_id = self
            .transaction_service
            .new_transaction(user_id, all_train_orders, all_atomic)
            .await
            .map_err(|e| {
                TrainOrderServiceError::InfrastructureError(ServiceError::RelatedServiceError(e.into()))
            })?;

        if let Err(e) = self
            .process_order_message(transaction_id, all_order_uuids, all_atomic)
            .await
        {
            error!("Failed to process orders immediately: {:?}", e);
            return Err(Box::new(e));
        }

        Ok(TransactionInfoDTO {
            transaction_id,
            amount: total_amount,
            status: "unpaid".to_string(),
        })
    }
}
