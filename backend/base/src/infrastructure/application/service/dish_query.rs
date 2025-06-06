use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::domain::Identifiable;
use crate::domain::service::station::StationService;
use crate::domain::service::train_type::{
    TrainTypeConfigurationService, TrainTypeConfigurationServiceError,
};
use crate::{
    application::{
        ApplicationError, GeneralError,
        commands::dish_query::DishQueryDTO,
        service::dish_query::{
            DishInfoDTO, DishQueryService, TakeawayDTO, TakeawayDishInfoDTO, TrainDishInfoDTO,
        },
    },
    domain::{
        model::{session::SessionId, train::TrainNumber},
        repository::{
            dish::DishRepository, takeaway::TakeawayShopRepository, train::TrainRepository,
        },
        service::{session::SessionManagerService, train_schedule::TrainScheduleService},
    },
};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use shared::utils::TimeMeter;
use std::collections::HashMap;

pub struct DishQueryServiceImpl<DR, TSR, TR, SMS, TSS, TTCS, SS>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TR: TrainRepository,
    SMS: SessionManagerService,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
    SS: StationService,
{
    dish_repository: Arc<DR>,
    takeaway_shop_repository: Arc<TSR>,
    train_repository: Arc<TR>,
    session_manager: Arc<SMS>,
    train_schedule_service: Arc<TSS>,
    train_type_configuration_service: Arc<TTCS>,
    station_service: Arc<SS>,
}

impl<DR, TSR, TR, SMS, TSS, TTCS, SS> DishQueryServiceImpl<DR, TSR, TR, SMS, TSS, TTCS, SS>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TR: TrainRepository,
    SMS: SessionManagerService,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
    SS: StationService,
{
    pub fn new(
        dish_repository: Arc<DR>,
        takeaway_shop_repository: Arc<TSR>,
        train_repository: Arc<TR>,
        session_manager: Arc<SMS>,
        train_schedule_service: Arc<TSS>,
        train_type_configuration_service: Arc<TTCS>,
        station_service: Arc<SS>,
    ) -> Self {
        DishQueryServiceImpl {
            dish_repository,
            takeaway_shop_repository,
            train_repository,
            session_manager,
            train_schedule_service,
            train_type_configuration_service,
            station_service,
        }
    }
}

#[async_trait]
impl<DR, TSR, TR, SMS, TSS, TTCS, SS> DishQueryService
    for DishQueryServiceImpl<DR, TSR, TR, SMS, TSS, TTCS, SS>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TR: TrainRepository,
    SMS: SessionManagerService,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
    SS: StationService,
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

        self.session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|e| {
                error!("Failed to get user id: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?
            .ok_or(Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        // 先假设车次经过了验证，然后查询是否存在，若不存在，则直接返回错误

        meter.meter("verify session");

        let train_number = self
            .train_type_configuration_service
            .verify_train_number(TrainNumber::from(query.train_number.clone()))
            .await
            .map_err(|e| match e {
                TrainTypeConfigurationServiceError::InfrastructureError(e) => {
                    error!("Infrastructure error while verifying train number: {:?}", e);
                    GeneralError::InternalServerError
                }
                _ => {
                    GeneralError::BadRequest(format!("invalid trainNumber: {}", query.train_number))
                }
            })?;

        let origin_departure_time = DateTimeWithTimeZone::parse_from_rfc3339(
            &query.origin_departure_time,
        )
        .map_err(|_for_super_earth| {
            GeneralError::BadRequest(format!(
                "invalid originDepartureTime: {}",
                query.origin_departure_time
            ))
        })?;

        meter.meter("verify train number");

        let terminal_arrival_time = self
            .train_schedule_service
            .get_terminal_arrival_time(train_number.clone(), origin_departure_time)
            .await
            .map_err(|e| {
                error!("Failed to get terminal arrival time: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        meter.meter("get terminal arrival time");

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
                    "/resource/dish/image/{}",
                    dish.images().first().unwrap_or(&Uuid::nil())
                ),
                price: dish.unit_price().to_f64().unwrap_or(0.0),
            })
            .collect::<Vec<_>>();

        meter.meter("transform dish");

        let train = self
            .train_repository
            .find_by_train_number(train_number)
            .await
            .map_err(|e| {
                error!("Failed to find train by number: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        meter.meter("load train");

        let route_id = train.default_route_id();

        let shop_by_stop = self
            .takeaway_shop_repository
            .find_by_train_route(route_id)
            .await
            .map_err(|e| {
                error!("Failed to find shops by train route: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        meter.meter("load takeaway shops");

        let mut takeaway_map = HashMap::new();

        let stations = self.station_service.get_stations().await.map_err(|e| {
            error!("Failed to get stations: {:?}", e);
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        meter.meter("load stations");

        let station_id_to_name = stations
            .into_iter()
            .map(|x| (x.get_id().unwrap(), x.name().to_string()))
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
                                "/resource/takeaway/image/{}",
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

        meter.meter("calculate");

        // 不能预订的话，直接就返回错误了？
        let can_booking = true;

        let reason = None;

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
