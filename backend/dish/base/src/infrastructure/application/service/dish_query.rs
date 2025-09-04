use async_trait::async_trait;
use shared::application_error::{ApplicationError, GeneralError};
use shared::internal::user::command::SessionQuery;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::application::commands::dish_query::DishQueryDTO;
use crate::application::service::dish_query::{
    DishInfoDTO, DishQueryService, TakeawayDTO, TakeawayDishInfoDTO, TrainDishInfoDTO,
};

use crate::domain::repository::{dish::DishRepository, takeaway::TakeawayShopRepository};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use shared::domain::model::session::SessionId;
use shared::domain::model::station::StationId;
use shared::domain::model::train::TrainNumber;
use shared::internal::order::command::VerifyTrainOrderQuery;
use shared::internal::train::command::{
    GetTerminalArrivalTimeQuery, GetTrainByNumberQuery, VerifyTrainNumberQuery,
};
use shared::ports::geo::GeoPort;
use shared::ports::order::OrderPort;
use shared::ports::train::TrainPort;
use shared::ports::user::UserPort;
use shared::utils::TimeMeter;
use std::collections::HashMap;

pub struct DishQueryServiceImpl<DR, TSR, GP, OP, TP, UP>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    GP: GeoPort,
    OP: OrderPort,
    TP: TrainPort,
    UP: UserPort,
{
    dish_repository: Arc<DR>,
    takeaway_shop_repository: Arc<TSR>,
    geo_port: Arc<GP>,
    order_port: Arc<OP>,
    train_port: Arc<TP>,
    user_port: Arc<UP>,
}

impl<DR, TSR, GP, OP, TP, UP> DishQueryServiceImpl<DR, TSR, GP, OP, TP, UP>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    GP: GeoPort,
    OP: OrderPort,
    TP: TrainPort,
    UP: UserPort,
{
    pub fn new(
        dish_repository: Arc<DR>,
        takeaway_shop_repository: Arc<TSR>,
        geo_port: Arc<GP>,
        order_port: Arc<OP>,
        train_port: Arc<TP>,
        user_port: Arc<UP>,
    ) -> Self {
        DishQueryServiceImpl {
            dish_repository,
            takeaway_shop_repository,
            geo_port,
            order_port,
            train_port,
            user_port,
        }
    }
}

#[async_trait]
impl<DR, TSR, GP, OP, TP, UP> DishQueryService for DishQueryServiceImpl<DR, TSR, GP, OP, TP, UP>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    GP: GeoPort,
    OP: OrderPort,
    TP: TrainPort,
    UP: UserPort,
{
    #[instrument(skip(self))]
    async fn query_dish(
        &self,
        query: DishQueryDTO,
        session_id: String,
    ) -> Result<TrainDishInfoDTO, Box<dyn ApplicationError>> {
        let mut meter = TimeMeter::new("query_dish");

        let session_id = SessionId::try_from(session_id.as_str())
            .map_err(|_| Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let user_id = self
            .user_port
            .get_session(SessionQuery {
                session_id: session_id.to_string(),
            })
            .await
            .map_err(|e| {
                error!("Failed to get session: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(GeneralError::InvalidSessionId)?
            .user_id;

        meter.meter("verify session");

        let train_number = self
            .train_port
            .get_train_by_number(GetTrainByNumberQuery {
                train_number: query.train_number.clone(),
            })
            .await
            .map_err(|e| {
                error!("Failed to get train: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or_else(|| {
                error!("Train number {} not found", query.train_number);
                GeneralError::NotFound(format!("Train number {} not found", query.train_number))
            })?
            .number;

        let origin_departure_time = DateTimeWithTimeZone::parse_from_rfc3339(
            &query.origin_departure_time,
        )
        .map_err(|_for_super_earth| {
            GeneralError::NotFound(format!(
                "invalid originDepartureTime: {}",
                query.origin_departure_time
            ))
        })?;

        meter.meter("verify train number");

        let terminal_arrival_time = self
            .train_port
            .get_terminal_arrival_time(GetTerminalArrivalTimeQuery {
                train_number: train_number.clone(),
                origin_departure_time,
            })
            .await
            .map_err(|e| {
                error!("Failed to get terminal arrival: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        meter.meter("get terminal arrival time");

        let train_number = if self
            .train_port
            .verify_train_number(VerifyTrainNumberQuery {
                train_number: train_number.clone(),
            })
            .await
            .inspect_err(|e| error!("Failed to verify train number: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
        {
            TrainNumber::from_unchecked(train_number)
        } else {
            return Err(
                GeneralError::NotFound(format!("Invalid train number: {}", train_number)).into(),
            );
        };

        let dishes = self
            .dish_repository
            .find_by_train_number(train_number.clone())
            .await
            .map_err(|e| {
                error!("Failed to find dishes by train number: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        meter.meter("load dish");

        let dish_dtos = dishes
            .into_iter()
            .map(|dish| DishInfoDTO {
                available_time: vec![dish.dish_time().to_string()],
                name: dish.name().to_string(),
                dish_type: dish.dish_type().to_string(),
                picture: format!(
                    "/resource/dish/images/{}",
                    dish.images().first().unwrap_or(&Uuid::nil())
                ),
                price: dish.unit_price().to_f64().unwrap_or(0.0),
            })
            .collect::<Vec<_>>();

        meter.meter("transform dish");

        let train = self
            .train_port
            .get_train_by_number(GetTrainByNumberQuery {
                train_number: train_number.clone().to_string(),
            })
            .await
            .map_err(|e| {
                error!("Failed to find train by number: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or_else(|| {
                error!("Train number {:?} not found", train_number);
                GeneralError::NotFound(format!("Train number {:?} not found", train_number))
            })?;

        meter.meter("load train");

        let route_id = train.default_route_id;

        let shop_by_stop = self
            .takeaway_shop_repository
            .find_by_train_route(route_id.into())
            .await
            .map_err(|e| {
                error!("Failed to find shops by train route: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        meter.meter("load takeaway shops");

        let mut takeaway_map = HashMap::new();

        let stations = self.geo_port.db_get_stations().await.map_err(|e| {
            error!("Failed to get stations: {:?}", e);
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        meter.meter("load stations");

        let station_id_to_name = stations
            .into_iter()
            .map(|x| (StationId::from(x.id as u64), x.name))
            .collect::<HashMap<_, _>>();

        for (stop, shops) in shop_by_stop {
            let station_id = stop.station_id();
            let station_name = station_id_to_name.get(&station_id).ok_or_else(|| {
                error!("Station ID {} not found in station list", station_id);
                GeneralError::InternalServerError
            })?;

            let shop_dtos = shops
                .into_iter()
                .map(|shop| {
                    let dish_dtos = shop
                        .dishes()
                        .iter()
                        .map(|dish| TakeawayDishInfoDTO {
                            name: dish.name().to_string(),
                            picture: format!(
                                "/resource/takeaway/images/{}",
                                dish.images().first().unwrap_or(&Uuid::nil())
                            ),
                            price: dish.unit_price().to_f64().unwrap_or(0.0),
                        })
                        .collect::<Vec<_>>();

                    TakeawayDTO {
                        shop_name: shop.name().to_string(),
                        dishes: dish_dtos,
                    }
                })
                .collect::<Vec<_>>();

            if !shop_dtos.is_empty() {
                takeaway_map.insert(station_name.to_string(), shop_dtos);
            }
        }

        meter.meter("check ticket ownership");

        let can_booking = self
            .order_port
            .verify_train_order(VerifyTrainOrderQuery {
                user_id,
                train_number: train_number.to_string(),
                origin_departure_time,
            })
            .await
            .map_err(|e| {
                error!("Failed to verify train order: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let reason = if can_booking {
            None
        } else {
            Some("您尚未购买此车次的火车票，无法预订餐食".to_string())
        };

        meter.meter("verify train order");

        info!("{}", meter);

        Ok(TrainDishInfoDTO {
            train_number: query.train_number,
            origin_departure_time: origin_departure_time.to_rfc3339(),
            terminal_arrival_time: terminal_arrival_time.to_rfc3339(),
            dishes: dish_dtos,
            takeaway: takeaway_map,
            can_booking,
            reason,
        })
    }
}
