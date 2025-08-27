use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::domain::service::order::OrderService;
use crate::domain::service::station::StationService;
use crate::domain::service::train_type::{
    TrainTypeConfigurationService, TrainTypeConfigurationServiceError,
};
use crate::domain::Identifiable;
use crate::{
    application::{
        commands::dish_query::DishQueryDTO, service::dish_query::{
            DishInfoDTO, DishQueryService, TakeawayDTO, TakeawayDishInfoDTO, TrainDishInfoDTO,
        },
        ApplicationError,
        GeneralError,
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

pub struct DishQueryServiceImpl<DR, TSR, TR, SMS, TSS, TTCS, SS, OS>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TR: TrainRepository,
    SMS: SessionManagerService,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
    SS: StationService,
    OS: OrderService,
{
    dish_repository: Arc<DR>,
    takeaway_shop_repository: Arc<TSR>,
    train_repository: Arc<TR>,
    session_manager: Arc<SMS>,
    train_schedule_service: Arc<TSS>,
    train_type_configuration_service: Arc<TTCS>,
    station_service: Arc<SS>,
    order_service: Arc<OS>,
}

impl<DR, TSR, TR, SMS, TSS, TTCS, SS, OS> DishQueryServiceImpl<DR, TSR, TR, SMS, TSS, TTCS, SS, OS>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TR: TrainRepository,
    SMS: SessionManagerService,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
    SS: StationService,
    OS: OrderService,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dish_repository: Arc<DR>,
        takeaway_shop_repository: Arc<TSR>,
        train_repository: Arc<TR>,
        session_manager: Arc<SMS>,
        train_schedule_service: Arc<TSS>,
        train_type_configuration_service: Arc<TTCS>,
        station_service: Arc<SS>,
        order_service: Arc<OS>,
    ) -> Self {
        DishQueryServiceImpl {
            dish_repository,
            takeaway_shop_repository,
            train_repository,
            session_manager,
            train_schedule_service,
            train_type_configuration_service,
            station_service,
            order_service,
        }
    }
}

#[async_trait]
impl<DR, TSR, TR, SMS, TSS, TTCS, SS, OS> DishQueryService
    for DishQueryServiceImpl<DR, TSR, TR, SMS, TSS, TTCS, SS, OS>
where
    DR: DishRepository,
    TSR: TakeawayShopRepository,
    TR: TrainRepository,
    SMS: SessionManagerService,
    TSS: TrainScheduleService,
    TTCS: TrainTypeConfigurationService,
    SS: StationService,
    OS: OrderService,
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
            .session_manager
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
                    "/resource/dish/images/{}",
                    dish.images().first().unwrap_or(&Uuid::nil())
                ),
                price: dish.unit_price().to_f64().unwrap_or(0.0),
            })
            .collect::<Vec<_>>();

        meter.meter("transform dish");

        let train = self
            .train_repository
            .find_by_train_number(train_number.clone())
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
            .order_service
            .verify_train_order(user_id, train_number.to_string(), origin_departure_time)
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use anyhow::anyhow;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use crate::application::commands::dish_query::DishQueryDTO;
    use crate::application::service::dish_query::DishQueryService;
    use crate::domain::model::dish::Dish;
    use crate::domain::model::dish::DishTime::Lunch;
    use crate::domain::model::train::Train;

    use crate::domain::model::train::{ TrainNumber, TrainType };

    use crate::domain::RepositoryError;

    use crate::domain::service::mock::{
        order::mock_order_service,
        session::mock_session_service,
        station::mock_station_service,
        train_schedule::mock_train_schedule_service,
        train_type::mock_train_type_service,
    };
    use crate::domain::repository::mock::{
        dish::mock_dish_repo,
        takeaway::mock_takeaway_shop_repo,
        train::mock_train_repo,
    };

    // ------------------- 正向测试 -------------------
    #[tokio::test]
    async fn test_query_dish_success() {
        // 1. Mock SessionManager 返回用户 ID
        let mut session_mock = mock_session_service();
        session_mock
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(1u64.into())));

        // 2. Mock TrainTypeConfigurationService 验证车次
        let mut train_type_mock = mock_train_type_service();
        train_type_mock
            .expect_verify_train_number()
            .returning(|train_number| Ok(TrainNumber::from_unchecked(train_number.to_string())));

        // 3. Mock TrainScheduleService 返回终点到达时间
        let mut train_schedule_mock = mock_train_schedule_service();
        train_schedule_mock
            .expect_get_terminal_arrival_time()
            .returning(|_, _| Ok(Utc::now().into()));

        // 4. Mock DishRepository 返回硬编码 Dish
        let mut dish_repo_mock = mock_dish_repo();
        dish_repo_mock
            .expect_find_by_train_number()
            .returning(|_| {
                Ok(vec![Dish::new(
                    Some(1u64.into()),
                    1u64.into(),
                    "饺子".to_string(),
                    Lunch,
                    "饺子炒饭".to_string(),
                    Decimal::new(100, 2),
                    vec![Uuid::new_v4()],
                )]) // 你需要在 Dish 模型里实现 new_example()
            });

        // 5. Mock TrainRepository 返回 Train
        let mut train_repo_mock = mock_train_repo();
        train_repo_mock
            .expect_find_by_train_number()
            .returning(|_| Ok(Train::new(
                Some(1u64.into()),
                TrainNumber::from_unchecked("G101".to_string()),
                TrainType::from_unchecked("G".to_string()),
                HashMap::new(),
                1u64.into(),
                1000,
            )));

        // 6. Mock TakeawayShopRepository 返回空
        let mut takeaway_repo_mock = mock_takeaway_shop_repo();
        takeaway_repo_mock
            .expect_find_by_train_route()
            .returning(|_| Ok(HashMap::new()));

        // 7. Mock StationService 返回空
        let mut station_service_mock = mock_station_service();
        station_service_mock
            .expect_get_stations()
            .returning(|| Ok(vec![]));

        // 8. Mock OrderService 返回可订餐
        let mut order_service_mock = mock_order_service();
        order_service_mock
            .expect_verify_train_order()
            .returning(|_, _, _| Ok(true));

        // 创建 Service
        let service = DishQueryServiceImpl::new(
            Arc::new(dish_repo_mock),
            Arc::new(takeaway_repo_mock),
            Arc::new(train_repo_mock),
            Arc::new(session_mock),
            Arc::new(train_schedule_mock),
            Arc::new(train_type_mock),
            Arc::new(station_service_mock),
            Arc::new(order_service_mock),
        );

        // 查询 DTO
        let query_dto = DishQueryDTO {
            train_number: "G101".to_string(),
            origin_departure_time: Utc::now().to_rfc3339(),
        };

        let result = service.query_dish(query_dto, Uuid::new_v4().to_string()).await;
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.train_number, "G101");
        assert!(dto.dishes.len() > 0);
        assert!(dto.can_booking);
    }

    // ------------------- 反向测试：无效 session -------------------
    #[tokio::test]
    async fn test_query_dish_invalid_session() {
        let mut session_mock = mock_session_service();
        session_mock
            .expect_get_user_id_by_session()
            .returning(|_| Ok(None));

        let service = DishQueryServiceImpl::new(
            Arc::new(mock_dish_repo()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_repo()),
            Arc::new(session_mock),
            Arc::new(mock_train_schedule_service()),
            Arc::new(mock_train_type_service()),
            Arc::new(mock_station_service()),
            Arc::new(mock_order_service()),
        );

        let query_dto = DishQueryDTO {
            train_number: "G101".to_string(),
            origin_departure_time: Utc::now().to_rfc3339(),
        };

        let result = service.query_dish(query_dto, "invalid_session".to_string()).await;
        assert!(result.is_err());
    }

    // ------------------- 反向测试：DishRepository 返回错误 -------------------
    #[tokio::test]
    async fn test_query_dish_dish_repo_error() {
        let mut session_mock = mock_session_service();
        session_mock
            .expect_get_user_id_by_session()
            .returning(|_| Ok(Some(1u64.into())));

        let mut dish_repo_mock = mock_dish_repo();
        dish_repo_mock
            .expect_find_by_train_number()
            .returning(|_| Err(RepositoryError::InconsistentState(anyhow!("error".to_string()))));

        let service = DishQueryServiceImpl::new(
            Arc::new(dish_repo_mock),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_train_repo()),
            Arc::new(session_mock),
            Arc::new(mock_train_schedule_service()),
            Arc::new(mock_train_type_service()),
            Arc::new(mock_station_service()),
            Arc::new(mock_order_service()),
        );

        let query_dto = DishQueryDTO {
            train_number: "G101".to_string(),
            origin_departure_time: Utc::now().to_rfc3339(),
        };

        let result = service.query_dish(query_dto, "session123".to_string()).await;
        assert!(result.is_err());
    }
}
