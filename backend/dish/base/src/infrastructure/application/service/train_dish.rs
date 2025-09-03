use crate::application::service::train_dish::TrainDishApplicationService;
use crate::domain::repository::dish::DishRepository;
use crate::domain::repository::takeaway::TakeawayShopRepository;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveTime};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use shared::Verified;
use shared::api::ApplicationError;
use shared::application_error::TrainOrderServiceError::InfrastructureError;
use shared::application_error::{GeneralError, TrainDishApplicationServiceError};
use shared::domain::model::dish::DishTime;
use shared::domain::model::order::{
    BaseOrder, DishOrder, Order, OrderId, OrderStatus, OrderTimeInfo, PaymentInfo, TakeawayOrder,
    TrainOrder,
};
use shared::domain::model::personal_info::PersonalInfoId;
use shared::domain::model::session::SessionId;
use shared::domain::model::station::StationId;
use shared::domain::model::takeaway::TakeawayDish;
use shared::domain::model::train::TrainNumber;
use shared::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use shared::domain::model::transaction::{Transaction, TransactionStatus};
use shared::domain::model::user::UserId;
use shared::domain::{Identifiable, Repository};
use shared::internal::dish::command::OrderTrainDishCommand;
use shared::internal::dish::dto::{
    DishOrderRequestDTO, TakeawayOrderRequestDTO, VerifiedDishOrderRequest,
    VerifiedTakeawayOrderRequest,
};
use shared::internal::order::dto::TransactionInfoDTO;
use shared::internal::user::command::{SessionQuery, UserInfoQuery};
use shared::ports::geo::GeoPort;
use shared::ports::order::OrderPort;
use shared::ports::train::TrainPort;
use shared::ports::user::UserPort;
use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct TrainDishApplicationServiceImpl<DR, TR, TP, UP, GP, OP>
where
    DR: DishRepository,
    TR: TakeawayShopRepository,
    TP: TrainPort,
    UP: UserPort,
    GP: GeoPort,
    OP: OrderPort,
{
    dish_repository: Arc<DR>,
    takeaway_shop_repository: Arc<TR>,
    train_port: Arc<TP>,
    user_port: Arc<UP>,
    geo_port: Arc<GP>,
    order_port: Arc<OP>,
    tz_offset_hour: u32,
}

impl<DR, TR, TP, UP, GP, OP> TrainDishApplicationServiceImpl<DR, TR, TP, UP, GP, OP>
where
    DR: DishRepository,
    TR: TakeawayShopRepository,
    TP: TrainPort,
    UP: UserPort,
    GP: GeoPort,
    OP: OrderPort,
{
    pub fn new(
        dish_repository: Arc<DR>,
        takeaway_shop_repository: Arc<TR>,
        train_port: Arc<TP>,
        user_port: Arc<UP>,
        geo_port: Arc<GP>,
        order_port: Arc<OP>,
        tz_offset_hour: u32,
    ) -> Self {
        Self {
            dish_repository,
            takeaway_shop_repository,
            train_port,
            user_port,
            geo_port,
            order_port,
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
            .train_port
            .get_train_by_number(train_number.clone())
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
            .order_port
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
impl<DR, TR, TP, UP, GP, OP> TrainDishApplicationService
    for TrainDishApplicationServiceImpl<DR, TR, TP, UP, GP, OP>
where
    DR: DishRepository,
    TR: TakeawayShopRepository,
    TP: TrainPort,
    UP: UserPort,
    GP: GeoPort,
    OP: OrderPort,
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
            .user_port
            .get_session(SessionQuery {
                session_id: command.session_id,
            })
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        let train_number = TrainNumber::from(command.info.train_number.clone());

        let train_number = self
            .train_port
            .verify_train_number(train_number)
            .await
            .map_err(|e| match e {
                InfrastructureError(e) => {
                    error!("failed to verify train number: {}", e);
                    GeneralError::InternalServerError
                }
                _ => {
                    error!("failed to verify train number: {}", e);
                    GeneralError::NotFound(format!("invalid train number: {}", e))
                }
            })?;
        let personal_info_list = self
            .user_port
            .get_user_info(UserInfoQuery {
                user_id: user_id.user_id,
            })
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
            .train_port
            .find_by_train_number(train_number.clone())
            .await
            .inspect_err(|e| error!("failed to find train by number: {}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let train_schedule = self
            .train_port
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
            .geo_port
            .get_stations()
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

        self.order_port
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
