use crate::Verified;
use crate::application::commands::train_dish::OrderTrainDishCommand;
use crate::application::service::train_dish::{
    DishOrderRequestDTO, TakeawayOrderRequestDTO, TrainDishApplicationService,
    TrainDishApplicationServiceError, VerifiedDishOrderRequest, VerifiedTakeawayOrderRequest,
};
use crate::application::service::transaction::TransactionInfoDTO;
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::dish::DishTime;
use crate::domain::model::order::{
    BaseOrder, DishOrder, Order, OrderId, OrderStatus, OrderTimeInfo, PaymentInfo, TakeawayOrder,
    TrainOrder,
};
use crate::domain::model::personal_info::PersonalInfoId;
use crate::domain::model::session::SessionId;
use crate::domain::model::station::StationId;
use crate::domain::model::takeaway::TakeawayDish;
use crate::domain::model::train::TrainNumber;
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::model::transaction::{Transaction, TransactionStatus};
use crate::domain::model::user::UserId;
use crate::domain::repository::dish::DishRepository;
use crate::domain::repository::personal_info::PersonalInfoRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::takeaway::TakeawayShopRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::train_type::{
    TrainTypeConfigurationService, TrainTypeConfigurationServiceError,
};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveTime};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct TrainDishApplicationServiceImpl<TTCS, DR, TR, TSR, T, PIR, SMS, SR, TXR>
where
    TTCS: TrainTypeConfigurationService,
    DR: DishRepository,
    TR: TakeawayShopRepository,
    TSR: TrainScheduleRepository,
    T: TrainRepository,
    PIR: PersonalInfoRepository,
    SMS: SessionManagerService,
    SR: StationRepository,
    TXR: TransactionRepository,
{
    train_type_configuration_service: Arc<TTCS>,
    dish_repository: Arc<DR>,
    takeaway_shop_repository: Arc<TR>,
    train_schedule_repository: Arc<TSR>,
    train_repository: Arc<T>,
    personal_info_repository: Arc<PIR>,
    session_manager_service: Arc<SMS>,
    station_repository: Arc<SR>,
    transaction_repository: Arc<TXR>,
    tz_offset_hour: u32,
}

impl<TTCS, DR, TR, TSR, T, PIR, SMS, SR, TXR>
    TrainDishApplicationServiceImpl<TTCS, DR, TR, TSR, T, PIR, SMS, SR, TXR>
where
    TTCS: TrainTypeConfigurationService,
    DR: DishRepository,
    TR: TakeawayShopRepository,
    TSR: TrainScheduleRepository,
    T: TrainRepository,
    PIR: PersonalInfoRepository,
    SMS: SessionManagerService,
    SR: StationRepository,
    TXR: TransactionRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        train_type_configuration_service: Arc<TTCS>,
        dish_repository: Arc<DR>,
        takeaway_shop_repository: Arc<TR>,
        train_schedule_repository: Arc<TSR>,
        train_repository: Arc<T>,
        personal_info_repository: Arc<PIR>,
        session_manager_service: Arc<SMS>,
        station_repository: Arc<SR>,
        transaction_repository: Arc<TXR>,
        tz_offset_hour: u32,
    ) -> Self {
        Self {
            train_type_configuration_service,
            dish_repository,
            takeaway_shop_repository,
            train_schedule_repository,
            train_repository,
            personal_info_repository,
            session_manager_service,
            station_repository,
            transaction_repository,
            tz_offset_hour,
        }
    }

    async fn verify_dish_order_request(
        &self,
        train_number: TrainNumber<Verified>,
        origin_departure_time: String,
        personal_uuid_to_id: &HashMap<Uuid, PersonalInfoId>,
        request_list: Vec<DishOrderRequestDTO>,
    ) -> Result<Vec<VerifiedDishOrderRequest>, Box<dyn ApplicationError>> {
        let origin_departure_time =
            DateTime::parse_from_rfc3339(&origin_departure_time).map_err(|e| {
                GeneralError::BadRequest(format!("invalid origin departure time: {}", e))
            })?;

        let train = self
            .train_repository
            .find_by_train_number(train_number.clone())
            .await
            .inspect_err(|e| error!("failed to find train by number: {}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let dish_list = self
            .dish_repository
            .find_by_train_number(train_number)
            .await
            .inspect_err(|e| error!("failed to find dishes for train: {}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let dish_name_to_dish = dish_list
            .into_iter()
            .map(|dish| (dish.name().to_string(), dish))
            .collect::<HashMap<_, _>>();

        let mut result = Vec::with_capacity(request_list.len());

        for request in request_list {
            let requested_dish = dish_name_to_dish.get(&request.name).ok_or_else(|| {
                TrainDishApplicationServiceError::InvalidDishName(request.name.clone())
            })?;

            let personal_info_id = *personal_uuid_to_id.get(&request.personal_id).ok_or({
                GeneralError::BadRequest(format!(
                    "invalid personal info ID: {}",
                    request.personal_id
                ))
            })?;

            let amount = Decimal::from(request.amount);

            // `request.amount >= 0`，故只需检查是否为零
            if amount == Decimal::ZERO {
                return Err(Box::new(GeneralError::BadRequest(format!(
                    "invalid dish amount: {}",
                    amount
                ))) as Box<dyn ApplicationError>);
            }

            let dish_time =
                DishTime::try_from(request.dish_time.as_str()).map_err(|_for_super_earth| {
                    GeneralError::BadRequest(format!("invalid dish time: {}", request.dish_time))
                })?;

            let active_hour: u32 = match dish_time {
                DishTime::Lunch => 12,
                DishTime::Dinner => 18,
            };

            let active_time = origin_departure_time
                .with_time(NaiveTime::from_hms_opt(active_hour, 0, 0).unwrap())
                .unwrap();

            result.push(VerifiedDishOrderRequest {
                dish_id: requested_dish.get_id().expect("dish should have an ID"),
                personal_id: personal_info_id,
                train_id: train.get_id().expect("train should have an ID"),
                unit_price: requested_dish.unit_price(),
                amount,
                dish_time,
                active_time,
            })
        }

        Ok(result)
    }

    #[instrument(skip(self, personal_uuid_to_id))]
    async fn verify_takeaway_order_request(
        &self,
        train_schedule: &TrainSchedule,
        personal_uuid_to_id: &HashMap<Uuid, PersonalInfoId>,
        station_name_to_id: &HashMap<String, StationId>,
        request_list: Vec<TakeawayOrderRequestDTO>,
    ) -> Result<Vec<VerifiedTakeawayOrderRequest>, Box<dyn ApplicationError>> {
        let takeaway_shops_map = self
            .takeaway_shop_repository
            .find_by_train_route(train_schedule.route_id())
            .await
            .inspect_err(|e| error!("failed to find takeaway shops for train route: {}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let mut station_id_to_shop_name_to_takeaway_name_to_takeaway: HashMap<
            StationId,
            HashMap<String, HashMap<String, TakeawayDish>>,
        > = HashMap::new();

        let mut station_id_to_arrival_time: HashMap<StationId, u32> = HashMap::new();

        for (stop, takeaway_shop_list) in takeaway_shops_map {
            station_id_to_arrival_time.insert(stop.station_id(), stop.arrival_time());

            let station_entry = station_id_to_shop_name_to_takeaway_name_to_takeaway
                .entry(stop.station_id())
                .or_default();

            for takeaway_shop in takeaway_shop_list {
                let shop_entry = station_entry
                    .entry(takeaway_shop.name().to_string())
                    .or_default();

                for dish in takeaway_shop.dishes() {
                    shop_entry.insert(dish.name().to_string(), dish.clone());
                }
            }
        }

        let mut result = Vec::with_capacity(request_list.len());

        for request in request_list {
            let shop_name = request.shop_name.clone();
            let takeaway_name = request.name.clone();

            let station_id = station_name_to_id.get(&request.station).ok_or_else(|| {
                TrainDishApplicationServiceError::InvalidTakeawayStation(request.station.clone())
            })?;

            let shop_name_to_takeaway_name_to_takeaway =
                station_id_to_shop_name_to_takeaway_name_to_takeaway
                    .get(station_id)
                    .ok_or_else(|| {
                        TrainDishApplicationServiceError::InvalidTakeawayStation(
                            request.station.clone(),
                        )
                    })?;

            let shop_entry = shop_name_to_takeaway_name_to_takeaway
                .get(&shop_name)
                .ok_or_else(|| {
                    TrainDishApplicationServiceError::InvalidTakeawayShopName(shop_name.clone())
                })?;

            let requested_dish = shop_entry.get(&takeaway_name).ok_or_else(|| {
                TrainDishApplicationServiceError::InvalidDishName(takeaway_name.clone())
            })?;

            let personal_info_id = *personal_uuid_to_id.get(&request.personal_id).ok_or({
                GeneralError::BadRequest(format!(
                    "invalid personal info ID: {}",
                    request.personal_id
                ))
            })?;

            let station_arrival_time =
                *station_id_to_arrival_time.get(station_id).ok_or_else(|| {
                    error!(
                        "inconsistent state: station arrival time not found for station ID: {:?}",
                        station_id
                    );
                    GeneralError::InternalServerError
                })?;

            let active_time = train_schedule
                .date()
                .and_time(
                    NaiveTime::from_num_seconds_from_midnight_opt(
                        train_schedule.origin_departure_time() as u32 + station_arrival_time,
                        0,
                    )
                    .unwrap(),
                )
                .and_local_timezone(
                    FixedOffset::east_opt(self.tz_offset_hour as i32 * 3600).unwrap(),
                )
                .unwrap();

            let amount = Decimal::from(request.amount);

            // `request.amount >= 0`，故只需检查是否为零
            if amount == Decimal::ZERO {
                return Err(Box::new(GeneralError::BadRequest(format!(
                    "invalid takeaway amount: {}",
                    amount
                ))) as Box<dyn ApplicationError>);
            }

            result.push(VerifiedTakeawayOrderRequest {
                takeaway_dish_id: requested_dish.get_id().expect("dish should have an ID"),
                train_id: train_schedule.train_id(),
                station_id: *station_id,
                personal_id: personal_info_id,
                unit_price: requested_dish.unit_price(),
                amount,
                active_time,
            });
        }

        Ok(result)
    }

    async fn verify_train_order(
        &self,
        user_id: UserId,
        train_schedule_id: TrainScheduleId,
    ) -> Result<OrderId, Box<dyn ApplicationError>> {
        let user_transaction_list = self
            .transaction_repository
            .find_by_user_id(user_id)
            .await
            .inspect_err(|e| {
                error!("failed to find transactions by user ID: {}", e);
            })
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let related_order = user_transaction_list
            .iter()
            .filter(|tx| tx.status() == TransactionStatus::Paid)
            .flat_map(|tx| tx.orders())
            .filter(|order| {
                order.order_status() == OrderStatus::Paid
                    || order.order_status() == OrderStatus::Ongoing
                    || order.order_status() == OrderStatus::Active
            })
            .find(|order| {
                ((*order).deref() as &dyn Any)
                    .downcast_ref::<TrainOrder>()
                    .is_some_and(|train_order| train_order.train_schedule_id() == train_schedule_id)
            })
            .ok_or(TrainDishApplicationServiceError::NoRelatedTrainOrder)?;

        Ok(related_order.order_id().expect("order should have an ID"))
    }
}

#[async_trait]
impl<TTCS, DR, TR, TSR, T, PIR, SMS, SR, TXR> TrainDishApplicationService
    for TrainDishApplicationServiceImpl<TTCS, DR, TR, TSR, T, PIR, SMS, SR, TXR>
where
    TTCS: TrainTypeConfigurationService,
    DR: DishRepository,
    TR: TakeawayShopRepository,
    TSR: TrainScheduleRepository,
    T: TrainRepository,
    PIR: PersonalInfoRepository,
    SMS: SessionManagerService,
    SR: StationRepository,
    TXR: TransactionRepository,
{
    async fn order_dish(
        &self,
        command: OrderTrainDishCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        let session_id = SessionId::from(
            Uuid::try_parse(&command.session_id)
                .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?,
        );

        let user_id = self
            .session_manager_service
            .get_user_id_by_session(session_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        let train_number = TrainNumber::from(command.info.train_number.clone());

        let train_number = self
            .train_type_configuration_service
            .verify_train_number(train_number)
            .await
            .map_err(|e| match e {
                TrainTypeConfigurationServiceError::InfrastructureError(e) => {
                    error!("failed to verify train number: {}", e);
                    GeneralError::InternalServerError
                }
                _ => {
                    error!("failed to verify train number: {}", e);
                    GeneralError::NotFound(format!("invalid train number: {}", e))
                }
            })?;
        let personal_info_list = self
            .personal_info_repository
            .find_by_user_id(user_id)
            .await
            .inspect_err(|e| error!("failed to find personal info by user ID: {}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let personal_uuid_to_id: HashMap<Uuid, PersonalInfoId> = personal_info_list
            .into_iter()
            .map(|info| {
                (
                    info.uuid(),
                    info.get_id().expect("personal info should have an ID"),
                )
            })
            .collect();

        let origin_departure_time =
            DateTime::parse_from_rfc3339(&command.info.origin_departure_time).map_err(|e| {
                GeneralError::BadRequest(format!("invalid origin departure time: {}", e))
            })?;

        let train = self
            .train_repository
            .find_by_train_number(train_number.clone())
            .await
            .inspect_err(|e| error!("failed to find train by number: {}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let train_schedule = self
            .train_schedule_repository
            .find_by_train_id_and_origin_departure_time(
                train.get_id().expect("train should have an ID"),
                origin_departure_time,
            )
            .await
            .inspect_err(|e| error!("failed to find train schedule: {}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or_else(|| {
                // 车次号一定存在，但车次（包含出发时间）不一定存在

                GeneralError::NotFound(format!(
                    "no train schedule found for train number {} at {}",
                    command.info.train_number, command.info.origin_departure_time
                ))
            })?;

        let train_order_id = self
            .verify_train_order(
                user_id,
                train_schedule
                    .get_id()
                    .expect("train schedule should have an ID"),
            )
            .await?;

        let verified_dish_order_requests = self
            .verify_dish_order_request(
                train_number.clone(),
                command.info.origin_departure_time,
                &personal_uuid_to_id,
                command.info.dishes,
            )
            .await
            .inspect_err(|e| error!("failed to verify dish order request: {}", e))?;

        let station_list = self
            .station_repository
            .load()
            .await
            .inspect_err(|e| {
                error!("failed to load stations: {}", e);
            })
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let station_name_to_id: HashMap<String, StationId> = station_list
            .into_iter()
            .map(|station| {
                (
                    station.name().to_string(),
                    station.get_id().expect("station should have an ID"),
                )
            })
            .collect();

        let verified_takeaway_order_requests = self
            .verify_takeaway_order_request(
                &train_schedule,
                &personal_uuid_to_id,
                &station_name_to_id,
                command.info.takeaway,
            )
            .await
            .inspect_err(|e| error!("failed to verify takeaway order request: {}", e))?;

        let mut order_list: Vec<Box<dyn Order>> = Vec::new();

        for dish_request in verified_dish_order_requests {
            let order_time_info = OrderTimeInfo::new(
                Transaction::now(),
                dish_request.active_time,
                dish_request.active_time,
            );

            let base_order = BaseOrder::new(
                None,
                Uuid::new_v4(),
                OrderStatus::Unpaid,
                order_time_info,
                dish_request.unit_price,
                dish_request.amount,
                PaymentInfo::new(None, None),
                dish_request.personal_id,
            );

            let dish_order = DishOrder::new(
                base_order,
                train_order_id,
                dish_request.dish_id,
                dish_request.unit_price,
                dish_request.amount,
            );

            order_list.push(Box::new(dish_order))
        }

        for takeaway_request in verified_takeaway_order_requests {
            let order_time_info = OrderTimeInfo::new(
                Transaction::now(),
                takeaway_request.active_time,
                takeaway_request.active_time,
            );

            let base_order = BaseOrder::new(
                None,
                Uuid::new_v4(),
                OrderStatus::Unpaid,
                order_time_info,
                takeaway_request.unit_price,
                takeaway_request.amount,
                PaymentInfo::new(None, None),
                takeaway_request.personal_id,
            );

            let takeaway_order = TakeawayOrder::new(
                base_order,
                train_order_id,
                takeaway_request.takeaway_dish_id,
                takeaway_request.unit_price,
                takeaway_request.amount,
            );

            order_list.push(Box::new(takeaway_order));
        }

        let mut tx = Transaction::new(user_id, order_list, false);

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| {
                error!("failed to save transaction: {}", e);
            })
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(TransactionInfoDTO {
            transaction_id: tx.uuid(),
            amount: tx.amount().to_f64().unwrap(),
            status: tx.status().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::application::service::train_dish::TakeawayOrderRequestDTO;
    use crate::domain::model::order::{BaseOrder, OrderTimeInfo, PaymentInfo, TrainOrder};
    use crate::domain::model::personal_info::PreferredSeatLocation;
    use crate::domain::model::route::Stop;
    use crate::domain::model::takeaway::TakeawayDish;
    use crate::domain::model::train::{SeatType, SeatTypeName};
    use crate::domain::model::train_schedule::{
        Seat, SeatLocationInfo, SeatStatus, StationRange, TrainSchedule,
    };
    use crate::domain::model::transaction::TransactionStatus;
    use crate::domain::repository::mock::takeaway::{
        MockTakeawayShopRepository, mock_takeaway_shop_repo,
    };
    use crate::domain::repository::mock::transaction::MockTransactionRepository;
    use crate::domain::repository::mock::{
        dish::mock_dish_repo, personal_info::mock_personal_info_repository,
        station::mock_station_repository, train::mock_train_repo,
        train_schedule::mock_train_schedule_repository, transaction::mock_transaction_repository,
    };
    use crate::domain::service::mock::{
        session::mock_session_service, train_type::mock_train_type_service,
    };
    use crate::infrastructure::application::service::train_dish::TrainDishApplicationServiceImpl;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;
    //
    use crate::application::service::train_dish::DishOrderRequestDTO;
    use crate::domain::RepositoryError;
    use crate::domain::model::dish::{Dish, DishTime};
    use crate::domain::model::route::RouteId;
    use crate::domain::model::train::{Train, TrainNumber, TrainType};
    use crate::domain::model::user::UserId;
    use crate::domain::repository::mock::dish::MockDishRepository;
    use crate::domain::repository::mock::personal_info::MockPersonalInfoRepository;
    use crate::domain::repository::mock::station::MockStationRepository;
    use crate::domain::repository::mock::train::MockTrainRepository;
    use crate::domain::repository::mock::train_schedule::MockTrainScheduleRepository;
    use crate::domain::repository::mock::transaction::MockTransactionRepository as TxRepoMock;
    use crate::domain::service::ServiceError;
    use crate::domain::service::mock::session::MockSessionManagerService;
    use crate::domain::service::mock::train_type::MockTrainTypeConfigurationService;
    use crate::domain::service::train_type::TrainTypeConfigurationServiceError;
    use anyhow::anyhow;
    // bring command type into scope for order_dish tests
    use crate::application::commands::train_dish::OrderTrainDishCommand;
    use crate::application::service::train_dish::TrainDishApplicationService;
    use crate::domain::model::order::OrderStatus;

    // ============ verify_dish_order_request: 成功与失败用例 ============

    #[tokio::test]
    async fn test_verify_dish_order_request_success() {
        // 构造 train_repo 返回一个带ID的 Train
        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find_by_train_number().returning(|tn| {
            let seats = HashMap::new();
            let train = Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                seats,
                RouteId::from(1u64),
                0,
            );
            Ok(train)
        });

        // 构造 dish_repo 返回一个匹配名字的菜品
        let mut dish_repo = MockDishRepository::new();
        dish_repo.expect_find_by_train_number().returning(|_| {
            Ok(vec![Dish::new(
                Some(1u64.into()),
                1u64.into(),
                "主食".to_string(),
                DishTime::Lunch,
                "米饭".to_string(),
                Decimal::new(1500, 2),
                vec![],
            )])
        });

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(dish_repo),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(train_repo),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );

        let personal_uuid = Uuid::new_v4();
        let mut personal_map = HashMap::new();
        personal_map.insert(personal_uuid, 1u64.into());

        let reqs = vec![DishOrderRequestDTO {
            name: "米饭".to_string(),
            personal_id: personal_uuid,
            amount: 1,
            dish_time: "lunch".to_string(),
        }];

        let result = service
            .verify_dish_order_request(
                TrainNumber::from_unchecked("G123".to_string()),
                "2025-08-26T08:00:00+08:00".to_string(),
                &personal_map,
                reqs,
            )
            .await;

        assert!(result.is_ok());
        let verified = result.unwrap();
        assert_eq!(verified.len(), 1);
        assert_eq!(verified[0].amount, Decimal::ONE);
        assert_eq!(verified[0].unit_price, Decimal::new(1500, 2));
    }

    #[tokio::test]
    async fn test_verify_dish_order_request_fail_invalid_origin_time() {
        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find_by_train_number().returning(|tn| {
            let train = Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                RouteId::from(1u64),
                0,
            );
            Ok(train)
        });
        let mut dish_repo = MockDishRepository::new();
        dish_repo
            .expect_find_by_train_number()
            .returning(|_| Ok(Vec::new()));

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(dish_repo),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(train_repo),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );
        let mut personal_map = HashMap::new();
        personal_map.insert(Uuid::new_v4(), 1u64.into());

        let result = service
            .verify_dish_order_request(
                TrainNumber::from_unchecked("G123".to_string()),
                "not-a-time".to_string(),
                &personal_map,
                vec![],
            )
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 400);
    }

    #[tokio::test]
    async fn test_verify_dish_order_request_fail_invalid_dish_name() {
        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find_by_train_number().returning(|tn| {
            Ok(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                RouteId::from(1u64),
                0,
            ))
        });

        let mut dish_repo = MockDishRepository::new();
        dish_repo.expect_find_by_train_number().returning(|_| {
            Ok(vec![Dish::new(
                Some(2u64.into()),
                1u64.into(),
                "主食".to_string(),
                DishTime::Lunch,
                "牛肉面".to_string(),
                Decimal::new(2500, 2),
                vec![],
            )])
        });

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(dish_repo),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(train_repo),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );

        let pid = Uuid::new_v4();
        let mut personal_map = HashMap::new();
        personal_map.insert(pid, 1u64.into());
        let reqs = vec![DishOrderRequestDTO {
            name: "不存在的菜".to_string(),
            personal_id: pid,
            amount: 1,
            dish_time: "lunch".to_string(),
        }];

        let result = service
            .verify_dish_order_request(
                TrainNumber::from_unchecked("G123".to_string()),
                "2025-08-26T08:00:00+08:00".to_string(),
                &personal_map,
                reqs,
            )
            .await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        // 业务错误：InvalidDishName => 22001
        assert_eq!(err.error_code(), 22001);
    }

    #[tokio::test]
    async fn test_verify_dish_order_request_fail_personal_id_not_found() {
        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find_by_train_number().returning(|tn| {
            Ok(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                RouteId::from(1u64),
                0,
            ))
        });
        let mut dish_repo = MockDishRepository::new();
        dish_repo.expect_find_by_train_number().returning(|_| {
            Ok(vec![Dish::new(
                Some(3u64.into()),
                1u64.into(),
                "主食".to_string(),
                DishTime::Lunch,
                "米饭".to_string(),
                Decimal::new(1000, 2),
                vec![],
            )])
        });
        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(dish_repo),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(train_repo),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );

        let reqs = vec![DishOrderRequestDTO {
            name: "米饭".to_string(),
            personal_id: Uuid::new_v4(), // 不在 map 中
            amount: 1,
            dish_time: "lunch".to_string(),
        }];
        let personal_map = HashMap::new();
        let result = service
            .verify_dish_order_request(
                TrainNumber::from_unchecked("G123".to_string()),
                "2025-08-26T08:00:00+08:00".to_string(),
                &personal_map,
                reqs,
            )
            .await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 400);
    }

    #[tokio::test]
    async fn test_verify_dish_order_request_fail_invalid_dish_time() {
        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find_by_train_number().returning(|tn| {
            Ok(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                RouteId::from(1u64),
                0,
            ))
        });
        let mut dish_repo = MockDishRepository::new();
        dish_repo.expect_find_by_train_number().returning(|_| {
            Ok(vec![Dish::new(
                Some(4u64.into()),
                1u64.into(),
                "主食".to_string(),
                DishTime::Lunch,
                "米饭".to_string(),
                Decimal::new(1000, 2),
                vec![],
            )])
        });
        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(dish_repo),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(train_repo),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );
        let pid = Uuid::new_v4();
        let mut personal_map = HashMap::new();
        personal_map.insert(pid, 1u64.into());
        let reqs = vec![DishOrderRequestDTO {
            name: "米饭".to_string(),
            personal_id: pid,
            amount: 1,
            dish_time: "breakfast".to_string(), // 非法
        }];
        let result = service
            .verify_dish_order_request(
                TrainNumber::from_unchecked("G123".to_string()),
                "2025-08-26T08:00:00+08:00".to_string(),
                &personal_map,
                reqs,
            )
            .await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 400);
    }

    #[tokio::test]
    async fn test_verify_dish_order_request_fail_zero_amount() {
        let mut train_repo = MockTrainRepository::new();
        train_repo.expect_find_by_train_number().returning(|tn| {
            Ok(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                RouteId::from(1u64),
                0,
            ))
        });
        let mut dish_repo = MockDishRepository::new();
        dish_repo.expect_find_by_train_number().returning(|_| {
            Ok(vec![Dish::new(
                Some(5u64.into()),
                1u64.into(),
                "主食".to_string(),
                DishTime::Lunch,
                "米饭".to_string(),
                Decimal::new(1000, 2),
                vec![],
            )])
        });
        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(dish_repo),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(train_repo),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );
        let pid = Uuid::new_v4();
        let mut personal_map = HashMap::new();
        personal_map.insert(pid, 1u64.into());
        let reqs = vec![DishOrderRequestDTO {
            name: "米饭".to_string(),
            personal_id: pid,
            amount: 0, // 0 => 非法
            dish_time: "lunch".to_string(),
        }];
        let result = service
            .verify_dish_order_request(
                TrainNumber::from_unchecked("G123".to_string()),
                "2025-08-26T08:00:00+08:00".to_string(),
                &personal_map,
                reqs,
            )
            .await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 400);
    }

    // 成功用例：verify_train_order 成功
    #[tokio::test]
    async fn test_verify_train_order_success() {
        use crate::domain::model::order::OrderStatus;
        use crate::domain::model::transaction::Transaction;

        let mut tx_repo = MockTransactionRepository::new();

        // 构造一个 TrainOrder，状态为 Paid
        let schedule_id = 1u64.into();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        let train_order = TrainOrder::new(
            base_order,
            1u64.into(),
            Some(Seat::new(
                1u64.into(),
                SeatType::new(
                    Some(1u64.into()),
                    SeatTypeName::from_unchecked("二等座".to_string()),
                    1,
                    Decimal::new(1000, 2),
                ),
                SeatLocationInfo {
                    carriage: 3,
                    row: 11,
                    location: 'A',
                },
                SeatStatus::Occupied,
            )),
            SeatTypeName::from_unchecked("二等座".to_string()),
            Some(PreferredSeatLocation::A),
            StationRange::from_unchecked(1u64.into(), 1u64.into()),
        );

        let transaction = Transaction::new_full(
            None,
            Uuid::new_v4(),
            Transaction::now(),
            Some(Transaction::now()),
            Decimal::ONE,
            TransactionStatus::Paid,
            1u64.into(),
            vec![Box::new(train_order)],
            false,
        );

        // mock 返回这个 Transaction
        tx_repo
            .expect_find_by_user_id()
            .returning(move |_| Ok(vec![transaction.clone()]));

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(tx_repo),
            8,
        );

        let user_id = 1u64.into();

        let result = service.verify_train_order(user_id, schedule_id).await;

        println!("{:?}", result);

        assert!(result.is_ok());
        let order_id = result.unwrap();
        assert_eq!(order_id, 1u64.into()); // 校验返回的就是我们构造的 order_id
    }

    // 失败用例：verify_train_order 无订单
    #[tokio::test]
    async fn test_verify_train_order_no_related_order() {
        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_find_by_user_id().returning(|_| Ok(vec![])); // 返回空，模拟无订单

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(tx_repo),
            8,
        );

        let user_id = 1u64.into();
        let schedule_id = 1u64.into();

        let result = service.verify_train_order(user_id, schedule_id).await;
        assert!(result.is_err()); // 这里预期失败
    }

    // 成功用例：verify_takeaway_order_request
    #[tokio::test]
    async fn test_verify_takeaway_order_request_success() {
        let mut takeaway_repo = MockTakeawayShopRepository::new();
        takeaway_repo.expect_find_by_train_route().returning(|_| {
            let mut map = HashMap::new();
            // 模拟 route 下有一个 station 和 shop
            use crate::domain::model::takeaway::TakeawayShop;

            let stop = Stop::new(Some(1u64.into()), Some(1u64.into()), 1u64.into(), 0, 0, 0); // 停靠时间 100 秒
            let mut shop = TakeawayShop::new("好吃的店".to_string(), 1u64.into());

            shop.add_dish(TakeawayDish::new(
                Some(1u64.into()),
                Some(1u64.into()),
                "米饭".to_string(),
                "主食".to_string(),
                Decimal::new(1000, 2),
                vec![],
            ));

            map.insert(stop, vec![shop]);
            Ok(map)
        });

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(mock_dish_repo()),
            Arc::new(takeaway_repo),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );

        // 构造参数
        let train_schedule = TrainSchedule::new(
            Some(1u64.into()),
            1u64.into(),
            NaiveDate::from_ymd_opt(2025, 1, 1).expect("date"),
            0, // 出发时间
            1u64.into(),
        );

        let mut personal_uuid_to_id = HashMap::new();
        let uuid = Uuid::new_v4();
        personal_uuid_to_id.insert(uuid, 1u64.into());

        let mut station_name_to_id = HashMap::new();
        station_name_to_id.insert("测试站".to_string(), 1u64.into());

        let request_list = vec![TakeawayOrderRequestDTO {
            personal_id: uuid,
            station: "测试站".to_string(),
            shop_name: "好吃的店".to_string(),
            name: "米饭".to_string(),
            amount: 1,
        }];

        let result = service
            .verify_takeaway_order_request(
                &train_schedule,
                &personal_uuid_to_id,
                &station_name_to_id,
                request_list,
            )
            .await;

        assert!(result.is_ok());
        let verified = result.unwrap();
        assert_eq!(verified.len(), 1);
        assert_eq!(verified[0].amount, Decimal::ONE);
    }

    // 失败用例：无效车站
    #[tokio::test]
    async fn test_verify_takeaway_order_request_fail_invalid_station() {
        let mut takeaway_repo = MockTakeawayShopRepository::new();
        takeaway_repo
            .expect_find_by_train_route()
            .returning(|_| Ok(HashMap::new())); // 没有车站

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(mock_dish_repo()),
            Arc::new(takeaway_repo),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );

        let train_schedule = TrainSchedule::new(
            Some(1u64.into()),
            1u64.into(),
            NaiveDate::from_ymd_opt(2025, 1, 1).expect("date"),
            0, // 出发时间
            1u64.into(),
        );

        let mut personal_uuid_to_id = HashMap::new();
        let uuid = Uuid::new_v4();
        personal_uuid_to_id.insert(uuid, 1u64.into());

        let mut station_name_to_id = HashMap::new();
        station_name_to_id.insert("测试站".to_string(), 1u64.into());

        let request_list = vec![TakeawayOrderRequestDTO {
            personal_id: uuid,
            station: "测试站".to_string(),
            shop_name: "好吃的店".to_string(),
            name: "米饭".to_string(),
            amount: 1,
        }];

        let result = service
            .verify_takeaway_order_request(
                &train_schedule,
                &personal_uuid_to_id,
                &station_name_to_id,
                request_list,
            )
            .await;

        assert!(result.is_err());
    }

    // ============ order_dish: 关键分支与成功用例 ============

    #[tokio::test]
    async fn test_order_dish_fail_invalid_session_format() {
        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(mock_session_service()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );
        let cmd = OrderTrainDishCommand {
            session_id: "not-a-uuid".to_string(),
            info: crate::application::service::train_dish::TrainDishOrderRequestDTO {
                train_number: "G123".to_string(),
                origin_departure_time: "2025-08-26T08:00:00+08:00".to_string(),
                dishes: vec![],
                takeaway: vec![],
            },
        };
        let result = service.order_dish(cmd).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 403);
    }

    #[tokio::test]
    async fn test_order_dish_fail_invalid_session_not_found() {
        let mut session = MockSessionManagerService::new();
        session
            .expect_get_user_id_by_session()
            .returning(|_| Ok(None));

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(mock_train_type_service()),
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(session),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );

        let cmd = OrderTrainDishCommand {
            session_id: Uuid::new_v4().to_string(),
            info: crate::application::service::train_dish::TrainDishOrderRequestDTO {
                train_number: "G123".to_string(),
                origin_departure_time: "2025-08-26T08:00:00+08:00".to_string(),
                dishes: vec![],
                takeaway: vec![],
            },
        };
        let result = service.order_dish(cmd).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 403);
    }

    #[tokio::test]
    async fn test_order_dish_fail_invalid_train_number_maps_not_found() {
        let mut session = MockSessionManagerService::new();
        session
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(UserId::from(1u64))));

        let mut tcs = MockTrainTypeConfigurationService::new();
        tcs.expect_verify_train_number().returning(|_| {
            Err(TrainTypeConfigurationServiceError::InvalidTrainNumber(
                "bad".into(),
            ))
        });

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(tcs),
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(session),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );
        let cmd = OrderTrainDishCommand {
            session_id: Uuid::new_v4().to_string(),
            info: crate::application::service::train_dish::TrainDishOrderRequestDTO {
                train_number: "GXXX".to_string(),
                origin_departure_time: "2025-08-26T08:00:00+08:00".to_string(),
                dishes: vec![],
                takeaway: vec![],
            },
        };
        let result = service.order_dish(cmd).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 404);
    }

    #[tokio::test]
    async fn test_order_dish_fail_train_number_infra_error_maps_500() {
        let mut session = MockSessionManagerService::new();
        session
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(UserId::from(1u64))));
        let mut tcs = MockTrainTypeConfigurationService::new();
        tcs.expect_verify_train_number().returning(|_| {
            Err(TrainTypeConfigurationServiceError::InfrastructureError(
                ServiceError::RepositoryError(RepositoryError::Db(anyhow!("db"))),
            ))
        });

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(tcs),
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_schedule_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_personal_info_repository()),
            Arc::new(session),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );
        let cmd = OrderTrainDishCommand {
            session_id: Uuid::new_v4().to_string(),
            info: crate::application::service::train_dish::TrainDishOrderRequestDTO {
                train_number: "G123".to_string(),
                origin_departure_time: "2025-08-26T08:00:00+08:00".to_string(),
                dishes: vec![],
                takeaway: vec![],
            },
        };
        let result = service.order_dish(cmd).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 500);
    }

    #[tokio::test]
    async fn test_order_dish_fail_no_schedule_found() {
        // session ok
        let mut session = MockSessionManagerService::new();
        session
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(1u64.into())));
        // verify train number ok
        let mut tcs = MockTrainTypeConfigurationService::new();
        tcs.expect_verify_train_number().returning(|tn| {
            Ok(crate::domain::model::train::TrainNumber::from_unchecked(
                tn.to_string(),
            ))
        });
        // personal info ok
        let mut pir = MockPersonalInfoRepository::new();
        pir.expect_find_by_user_id().returning(|_| Ok(Vec::new())); // 空也可以，后续 verify_* 会处理
        // train by number ok
        let mut tr = MockTrainRepository::new();
        tr.expect_find_by_train_number().returning(|tn| {
            Ok(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                RouteId::from(1u64),
                0,
            ))
        });
        // schedule not found => None
        let mut tsr = MockTrainScheduleRepository::new();
        tsr.expect_find_by_train_id_and_origin_departure_time()
            .returning(|_, _| Ok(None));

        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(tcs),
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(tsr),
            Arc::new(tr),
            Arc::new(pir),
            Arc::new(session),
            Arc::new(mock_station_repository()),
            Arc::new(mock_transaction_repository()),
            8,
        );
        let cmd = OrderTrainDishCommand {
            session_id: Uuid::new_v4().to_string(),
            info: crate::application::service::train_dish::TrainDishOrderRequestDTO {
                train_number: "G123".to_string(),
                origin_departure_time: "2025-08-26T08:00:00+08:00".to_string(),
                dishes: vec![],
                takeaway: vec![],
            },
        };
        let result = service.order_dish(cmd).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 404);
    }

    #[tokio::test]
    async fn test_order_dish_success_with_dish_only() {
        // session ok
        let mut session = MockSessionManagerService::new();
        session
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(1u64.into())));
        // verify train number ok
        let mut tcs = MockTrainTypeConfigurationService::new();
        tcs.expect_verify_train_number().returning(|tn| {
            Ok(crate::domain::model::train::TrainNumber::from_unchecked(
                tn.to_string(),
            ))
        });
        // personal info: 提供一个 personal uuid 映射
        use crate::domain::model::personal_info::{PersonalInfo, PreferredSeatLocation};
        use crate::domain::model::user::{IdentityCardId, RealName};
        use std::convert::TryFrom;
        let mut pir = MockPersonalInfoRepository::new();
        let personal_uuid = Uuid::new_v4();
        let personal = PersonalInfo::new(
            Some(1u64.into()),
            personal_uuid,
            RealName::try_from("张三".to_string()).unwrap(),
            IdentityCardId::try_from("11010519491231002X".to_string()).unwrap(),
            Some(PreferredSeatLocation::A),
            1u64.into(),
        );
        pir.expect_find_by_user_id()
            .returning(move |_| Ok(vec![personal.clone()]));

        // train ok
        let mut tr = MockTrainRepository::new();
        tr.expect_find_by_train_number().returning(|tn| {
            Ok(Train::new(
                Some(10u64.into()),
                TrainNumber::from_unchecked(tn.to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                RouteId::from(1u64),
                0,
            ))
        });
        // schedule ok
        let mut tsr = MockTrainScheduleRepository::new();
        let schedule = TrainSchedule::new(
            Some(100u64.into()),
            10u64.into(),
            NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
            0,
            1u64.into(),
        );
        tsr.expect_find_by_train_id_and_origin_departure_time()
            .returning(move |_, _| Ok(Some(schedule.clone())));

        // verify_train_order 需要历史已支付的 TrainOrder
        let mut tx_repo = TxRepoMock::new();
        use crate::domain::model::transaction::Transaction;
        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            OrderStatus::Paid,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );
        let train_order = TrainOrder::new(
            base_order,
            100u64.into(),
            None,
            SeatTypeName::from_unchecked("二等座".to_string()),
            None,
            StationRange::from_unchecked(1u64.into(), 1u64.into()),
        );
        let paid_tx = Transaction::new_full(
            None,
            Uuid::new_v4(),
            Transaction::now(),
            Some(Transaction::now()),
            Decimal::ONE,
            TransactionStatus::Paid,
            1u64.into(),
            vec![Box::new(train_order)],
            false,
        );
        tx_repo
            .expect_find_by_user_id()
            .returning(move |_| Ok(vec![paid_tx.clone()]));
        tx_repo.expect_save().returning(|_| Ok(1u64.into()));

        // dish repo: 提供一个匹配菜品
        let mut dish_repo = MockDishRepository::new();
        dish_repo.expect_find_by_train_number().returning(|_| {
            Ok(vec![Dish::new(
                Some(1u64.into()),
                10u64.into(),
                "主食".to_string(),
                DishTime::Lunch,
                "米饭".to_string(),
                Decimal::new(1500, 2),
                vec![],
            )])
        });

        // takeaway repo 也会被调用，返回空映射即可
        let mut takeaway_repo = MockTakeawayShopRepository::new();
        takeaway_repo
            .expect_find_by_train_route()
            .returning(|_| Ok(HashMap::new()))
            .times(1..);
        // station repo/load 会被调用，即便 takeaway 为空，也返回空列表即可
        let mut station_repo = MockStationRepository::new();
        station_repo
            .expect_load()
            .returning(|| Ok(vec![]))
            .times(0..);
        let service = TrainDishApplicationServiceImpl::new(
            Arc::new(tcs),
            Arc::new(dish_repo),
            Arc::new(takeaway_repo),
            Arc::new(tsr),
            Arc::new(tr),
            Arc::new(pir),
            Arc::new(session),
            Arc::new(station_repo),
            Arc::new(tx_repo),
            8,
        );

        let cmd = OrderTrainDishCommand {
            session_id: Uuid::new_v4().to_string(),
            info: crate::application::service::train_dish::TrainDishOrderRequestDTO {
                train_number: "G123".to_string(),
                origin_departure_time: "2025-08-26T08:00:00+08:00".to_string(),
                dishes: vec![DishOrderRequestDTO {
                    name: "米饭".to_string(),
                    personal_id: personal_uuid,
                    amount: 1,
                    dish_time: "lunch".to_string(),
                }],
                takeaway: vec![],
            },
        };

        let result = service.order_dish(cmd).await;
        if let Err(e) = &result {
            panic!(
                "order_dish failed: code={}, msg={}",
                e.error_code(),
                e.error_message()
            );
        }
        let dto = result.unwrap();
        assert!(dto.amount > 0.0);
        assert_eq!(dto.status, "unpaid");
    }
}
