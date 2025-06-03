//! TrainQueryServiceImpl 实现
//!
//! 该文件位于 `backend/base/src/infrastructure/application/service/train_query.rs`，
//! 负责把 HTTP / RPC 层收到的 CQRS 命令简单校验后转交给真正的
//! `TrainQueryService`（领域 / 应用服务）。
//!
//! * **为什么需要 `Arc<T>`?**
//!   应用服务通常是无状态、线程安全的，把它包进 `Arc` 便于在整个线程池里共享，
//!   无需再加 `Mutex`（不会修改内部状态）。
//! * **为什么要 `T: Send + Sync + 'static`?**
//!   Actix‑web、Tokio 等异步运行时会把服务对象跨线程移动，
//!   因此需要 `Send`；并发访问需要 `Sync`；`'static` 避免悬垂引用。

/// 火车查询应用服务实现
///
/// 此服务负责处理火车查询相关的应用逻辑，包括直达车次查询和中转车次查询。
/// 主要职责是验证输入参数，然后将请求委派给实际的领域服务处理。
///
/// # 功能
/// - 直达车次查询：根据出发站、目的站和出发日期查询直达火车
/// - 中转车次查询：根据出发城市、目的城市和出发日期查询需要换乘的火车方案
///
/// # 错误处理
/// - 输入验证：检查车站ID和城市ID是否为空
/// - 委托错误：传递领域服务可能返回的错误
///
/// # 实现细节
/// 该服务是对实际查询逻辑的一个适配层，主要负责：
/// 1. 参数校验 - 确保输入参数符合业务规则
/// 2. 错误转换 - 将领域错误映射为应用错误
/// 3. 转发调用 - 将实际查询工作委托给内部服务
// Step 1: Define `TrainQueryServiceImpl` application service implementation
// Step 2: Choose correct generics parameter according to data you need
// Exercise 1.2.1D - 5: Your code here. (1 / 6)
// HINT: You may refer to `UserManagerServiceImpl` for example
use std::sync::Arc;

use crate::application::commands::train_query::{
    DirectTrainQueryCommand, TrainQueryValidate, TrainScheduleQueryCommand,
    TransferTrainQueryCommand,
};
use crate::application::service::train_query::{
    DirectTrainQueryDTO, SeatInfoDTO, StoppingStationInfo, TrainInfoDTO, TrainQueryService,
    TrainQueryServiceError, TransferSolutionDTO, TransferTrainQueryDTO,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::station::StationId;
use crate::domain::model::train::TrainNumber;
use crate::domain::service::route::RouteService;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::station::StationService;
use crate::domain::service::train_schedule::TrainScheduleService;
use crate::domain::service::train_type::TrainTypeConfigurationService;
use async_trait::async_trait;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use std::collections::HashMap;
use tracing::{error, instrument};

// Thinking 1.2.1D - 4: 为何需要使用`+ 'static + Send + Sync`约束泛型参数？
// Thinking 1.2.1D - 5: 为何需要使用`Arc<T>`存储领域服务？为何无需使用`Arc<Mutex<T>>`？
pub struct TrainQueryServiceImpl<T, U, V, W, SMS>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
{
    // Step 3: Store service instance you need using `Arc<T>` and generics parameter
    // HINT: You may refer to `UserManagerServiceImpl` for example
    // Exercise 1.2.1D - 5: Your code here. (2 / 6)
    train_schedule_service: Arc<T>,
    station_service: Arc<U>,
    train_type_service: Arc<V>,
    route_service: Arc<W>,
    session_manager_service: Arc<SMS>,
}

// Step 4: Implement `new` associate function for `TrainQueryServiceImpl`
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (3 / 6)
impl<T, U, V, W, SMS> TrainQueryServiceImpl<T, U, V, W, SMS>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
{
    pub fn new(
        train_schedule_service: Arc<T>,
        station_service: Arc<U>,
        train_type_service: Arc<V>,
        route_service: Arc<W>,
        session_manager_service: Arc<SMS>,
    ) -> Self {
        TrainQueryServiceImpl {
            train_schedule_service,
            station_service,
            train_type_service,
            route_service,
            session_manager_service,
        }
    }

    async fn resolve_station_ids(
        &self,
        station_opt: &Option<String>,
        city_opt: &Option<String>,
    ) -> Result<Vec<StationId>, Box<dyn ApplicationError>> {
        match (station_opt, city_opt) {
            (Some(s), None) => {
                let id = self
                    .station_service
                    .get_station_by_name(s.trim().to_owned())
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by station name: {:?}", e);
                        GeneralError::InternalServerError
                    })?
                    .ok_or(TrainQueryServiceError::InvalidStationId)?
                    .get_id()
                    // SAFETY：get_station_by_name 返回的 Station 实例必定有 ID
                    .unwrap();
                Ok(vec![id])
            }
            (None, Some(city)) => {
                let stations = self
                    .station_service
                    .get_station_by_city_name(city)
                    .await
                    .map_err(|e| {
                        error!("Failed to get stations by city name: {:?}", e);
                        GeneralError::InternalServerError
                    })?;
                Ok(stations.into_iter().filter_map(|st| st.get_id()).collect())
            }
            _ => Err(Box::new(TrainQueryServiceError::InconsistentQuery)),
        }
    }

    async fn verify_session(&self, session_id: &str) -> Result<(), Box<dyn ApplicationError>> {
        if self
            .session_manager_service
            .verify_session_id(session_id)
            .await
            .inspect_err(|e| error!("Failed to verify session ID: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
        {
            return Err(Box::new(GeneralError::InvalidSessionId));
        }

        Ok(())
    }
}

// Step 5: Implement `TrainQueryService` trait for `TrainQueryServiceImpl`
// HINT: You need to use `async_trait` macro
// HINT: You should check user input in application service function,
// return error in validate failed
// HINT: You SHOULD NOT perform business logic in application service
// just delegate business logic to other service
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (4 / 6)
#[async_trait]
impl<T, U, V, W, SMS> TrainQueryService for TrainQueryServiceImpl<T, U, V, W, SMS>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
{
    #[instrument(skip(self))]
    async fn query_train(
        &self,
        cmd: TrainScheduleQueryCommand,
    ) -> Result<TrainInfoDTO, Box<dyn ApplicationError>> {
        self.verify_session(cmd.session_id.as_str()).await?;
        todo!()
    }
    #[instrument(skip(self))]
    async fn query_direct_trains(
        &self,
        cmd: DirectTrainQueryCommand,
    ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>> {
        self.verify_session(cmd.session_id.as_str()).await?;

        cmd.validate()?;

        let from_ids = self
            .resolve_station_ids(&cmd.departure_station, &cmd.departure_city)
            .await?;
        let to_ids = self
            .resolve_station_ids(&cmd.arrival_station, &cmd.arrival_city)
            .await?;

        let station_pairs: Vec<(StationId, StationId)> = from_ids
            .iter()
            .flat_map(|f| to_ids.iter().map(move |t| (*f, *t)))
            .collect();

        let schedules = self
            .train_schedule_service
            .direct_schedules(cmd.departure_time, &station_pairs)
            .await
            .map_err(|e| {
                error!("Failed to get direct schedules: {:?}", e);

                GeneralError::InternalServerError
            })?;

        let routes = self.route_service.get_routes().await.map_err(|e| {
            error!("Failed to get routes: {:?}", e);

            GeneralError::InternalServerError
        })?;
        let mut infos = Vec::new();
        for sch in schedules {
            infos.push(self.build_dto(&sch, &routes, cmd.departure_time).await?);
        }

        Ok(DirectTrainQueryDTO { solutions: infos })
    }

    #[instrument(skip(self))]
    async fn query_transfer_trains(
        &self,
        cmd: TransferTrainQueryCommand,
    ) -> Result<TransferTrainQueryDTO, Box<dyn ApplicationError>> {
        self.verify_session(cmd.session_id.as_str()).await?;
        cmd.validate()?;

        let from_ids = self
            .resolve_station_ids(&cmd.departure_station, &cmd.departure_city)
            .await?;
        let to_ids = self
            .resolve_station_ids(&cmd.arrival_station, &cmd.arrival_city)
            .await?;

        let station_pairs: Vec<(StationId, StationId)> = from_ids
            .iter()
            .flat_map(|f| to_ids.iter().map(move |t| (*f, *t)))
            .collect();

        let transfer_solutions = self
            .train_schedule_service
            .transfer_schedules(cmd.departure_time, &station_pairs)
            .await
            .map_err(|e| {
                error!("Failed to get transfer schedules: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        let routes = self.route_service.get_routes().await.map_err(|e| {
            error!("Failed to get routes: {:?}", e);
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        let mut solutions = Vec::new();
        for (schedule_ids, mid_station) in transfer_solutions {
            if schedule_ids.len() != 2 || mid_station.is_none() {
                continue;
            }

            let mut train_infos = Vec::new();
            for schedule_id in &schedule_ids {
                let schedules = self
                    .train_schedule_service
                    .get_schedules(cmd.departure_time)
                    .await
                    .map_err(|e| {
                        error!("Failed to get schedules: {:?}", e);
                        Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                    })?;

                let schedule = schedules.iter().find(|s| s.get_id() == Some(*schedule_id));

                if let Some(sch) = schedule {
                    train_infos.push(self.build_dto(sch, &routes, cmd.departure_time).await?);
                }
            }

            if train_infos.len() == 2 {
                let station_name = self
                    .station_service
                    .get_station_by_name(mid_station.unwrap().to_string())
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by id: {:?}", e);
                        Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                    })?
                    .map(|s| s.name().to_string())
                    .unwrap_or_else(|| "未知站点".to_string());

                let first_train_arrival_time = train_infos[0]
                    .route
                    .iter()
                    .find(|stop| stop.station_name == station_name)
                    .map(|stop| stop.arrival_time.clone())
                    .unwrap_or_else(|| train_infos[0].terminal_arrival_time.clone());

                let second_train_departure_time = train_infos[1]
                    .route
                    .iter()
                    .find(|stop| stop.station_name == station_name)
                    .map(|stop| stop.departure_time.clone())
                    .unwrap_or_else(|| train_infos[1].origin_departure_time.clone());

                let first_dt =
                    NaiveDateTime::parse_from_str(&first_train_arrival_time, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or_else(|_| cmd.departure_time.and_hms_opt(0, 0, 0).unwrap());
                let second_dt = NaiveDateTime::parse_from_str(
                    &second_train_departure_time,
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap_or_else(|_| cmd.departure_time.and_hms_opt(0, 0, 0).unwrap());

                let relaxing_time = if second_dt > first_dt {
                    (second_dt - first_dt).num_seconds() as u32
                } else {
                    (second_dt + Duration::hours(24) - first_dt).num_seconds() as u32
                };

                solutions.push(TransferSolutionDTO {
                    first_ride: train_infos[0].clone(),
                    second_ride: train_infos[1].clone(),
                    relaxing_time,
                });
            }
        }

        Ok(TransferTrainQueryDTO { solutions })
    }
}

impl<T, U, V, W, SMS> TrainQueryServiceImpl<T, U, V, W, SMS>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
{
    #[instrument(skip(self, routes))]
    async fn build_dto(
        &self,
        sch: &crate::domain::model::train_schedule::TrainSchedule,
        routes: &[crate::domain::model::route::Route],
        date: NaiveDate,
    ) -> Result<TrainInfoDTO, Box<dyn ApplicationError>> {
        // ——— 路线、停站 ———
        let route = routes
            .iter()
            .find(|r| r.get_id() == Some(sch.route_id()))
            .ok_or(TrainQueryServiceError::InvalidStationId)?;
        let mut stopping = Vec::<StoppingStationInfo>::new();
        for stop in route.stops() {
            let base = date.and_hms_opt(0, 0, 0).unwrap();
            let arr = (base + Duration::seconds(stop.arrival_time() as i64)).to_string();
            let dep = (base + Duration::seconds(stop.departure_time() as i64)).to_string();
            let name = self
                .station_service
                .get_station_by_name(stop.station_id().to_string())
                .await
                .map_err(|e| {
                    error!("Failed to get station by name: {:?}", e);

                    GeneralError::InternalServerError
                })?
                .map(|s| s.name().to_string())
                .ok_or_else(|| {
                    error!(
                        "Inconsistent: no station found for id {}",
                        stop.station_id()
                    );

                    GeneralError::InternalServerError
                })?;
            stopping.push(StoppingStationInfo {
                station_name: name,
                arrival_time: arr,
                departure_time: dep,
            });
        }

        // ——— 列车 / 座位 ———
        let train = self
            .train_type_service
            .get_train_by_number(TrainNumber::from_unchecked(sch.train_id().to_string()))
            .await
            .map_err(|e| {
                error!("Failed to get train by number: {:?}", e);

                GeneralError::InternalServerError
            })?;
        let mut seat_info = HashMap::new();
        for seat in train.seats().values() {
            seat_info.insert(
                seat.name().to_string(),
                SeatInfoDTO {
                    seat_type: seat.name().to_string(),
                    left: seat.capacity(),
                    price: seat.unit_price().to_string().parse().unwrap_or(0),
                },
            );
        }

        // ——— 其余字段 ———
        let dep_time = &stopping.first().unwrap().departure_time;
        let arr_time = &stopping.last().unwrap().arrival_time;
        let dep_dt = NaiveDateTime::parse_from_str(dep_time, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|_| date.and_hms_opt(0, 0, 0).unwrap());
        let arr_dt = NaiveDateTime::parse_from_str(arr_time, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|_| dep_dt + Duration::hours(2));

        Ok(TrainInfoDTO {
            departure_station: stopping.first().unwrap().station_name.clone(),
            departure_time: dep_time.clone(),
            arrival_station: stopping.last().unwrap().station_name.clone(),
            arrival_time: arr_time.clone(),
            origin_station: stopping.first().unwrap().station_name.clone(),
            origin_departure_time: stopping.first().unwrap().departure_time.clone(),
            terminal_station: stopping.last().unwrap().station_name.clone(),
            terminal_arrival_time: stopping.last().unwrap().arrival_time.clone(),
            train_number: train.number().to_string(),
            travel_time: (arr_dt - dep_dt).num_seconds() as u32,
            price: seat_info.values().map(|i| i.price).min().unwrap_or(0),
            route: stopping,
            seat_info,
        })
    }
}

// Step 6: Add unit test for your implementation
// HINT: You may use `mockall` crate to "mock" other service you depend on
// HINT: You may use AI tools to generate unit test
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (5 / 6)

// Step 7: Write document comment and mod comment for your implementation
// HINT: You may use AI tools to generate comment
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (6 / 6)

// Good! Next, register your application service in `api::main`
