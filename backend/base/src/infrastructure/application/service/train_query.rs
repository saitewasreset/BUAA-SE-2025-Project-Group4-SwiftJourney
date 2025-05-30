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
    DirectTrainQueryCommand,
    // TransferTrainQueryCommand,
};
use crate::application::service::train_query::{
    DirectTrainQueryDTO, SeatInfoDTO, StoppingStationInfo, TrainInfoDTO, TrainQueryService,
    TrainQueryServiceError,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::train::TrainNumber;
use crate::domain::service::route::RouteService;
use crate::domain::service::station::StationService;
use crate::domain::service::train_schedule::TrainScheduleService;
use crate::domain::service::train_type::TrainTypeConfigurationService;
use async_trait::async_trait;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use std::collections::HashMap;
use tracing::{error, instrument};

// Thinking 1.2.1D - 4: 为何需要使用`+ 'static + Send + Sync`约束泛型参数？
// Thinking 1.2.1D - 5: 为何需要使用`Arc<T>`存储领域服务？为何无需使用`Arc<Mutex<T>>`？
pub struct TrainQueryServiceImpl<T, U, V, W>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
{
    // Step 3: Store service instance you need using `Arc<T>` and generics parameter
    // HINT: You may refer to `UserManagerServiceImpl` for example
    // Exercise 1.2.1D - 5: Your code here. (2 / 6)
    train_schedule_service: Arc<T>,
    station_service: Arc<U>,
    train_type_service: Arc<V>,
    route_service: Arc<W>,
}

// Step 4: Implement `new` associate function for `TrainQueryServiceImpl`
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (3 / 6)
impl<T, U, V, W> TrainQueryServiceImpl<T, U, V, W>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
{
    pub fn new(
        train_schedule_service: Arc<T>,
        station_service: Arc<U>,
        train_type_service: Arc<V>,
        route_service: Arc<W>,
    ) -> Self {
        TrainQueryServiceImpl {
            train_schedule_service,
            station_service,
            train_type_service,
            route_service,
        }
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
impl<T, U, V, W> TrainQueryService for TrainQueryServiceImpl<T, U, V, W>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
{
    #[instrument(skip(self))]
    async fn query_direct_trains(
        &self,
        cmd: DirectTrainQueryCommand,
    ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>> {
        cmd.validate()?;

        let station_pairs = match (
            &cmd.departure_station,
            &cmd.arrival_station,
            &cmd.departure_city,
            &cmd.arrival_city,
        ) {
            // 情况1: 站点 -> 站点
            (Some(from_station), Some(to_station), None, None) => {
                let from_id = self
                    .station_service
                    .get_station_by_name(from_station.trim().to_string())
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by name: {:?}", e);

                        GeneralError::InternalServerError
                    })?
                    .ok_or(TrainQueryServiceError::InvalidStationId)?
                    .get_id()
                    .expect("Station should have an ID");
                let to_id = self
                    .station_service
                    .get_station_by_name(to_station.trim().to_string())
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by name: {:?}", e);

                        GeneralError::InternalServerError
                    })?
                    .ok_or(TrainQueryServiceError::InvalidStationId)?
                    .get_id()
                    .expect("Station should have an ID");
                vec![(from_id, to_id)]
            }

            // 情况2: 城市 -> 城市
            (None, None, Some(from_city), Some(to_city)) => self
                .station_service
                .station_pairs_by_city(from_city, to_city)
                .await
                .map_err(|e| {
                    error!("Failed to get station by name: {:?}", e);

                    GeneralError::InternalServerError
                })?,

            // 情况3: 站点 -> 城市
            (Some(from_station), None, None, Some(to_city)) => {
                let from_id = self
                    .station_service
                    .get_station_by_name(from_station.trim().to_string())
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by name: {:?}", e);

                        GeneralError::InternalServerError
                    })?
                    .ok_or(TrainQueryServiceError::InvalidStationId)?
                    .get_id()
                    .expect("Station should have an ID");

                let to_stations = self
                    .station_service
                    .get_station_by_city_name(to_city)
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by name: {:?}", e);

                        GeneralError::InternalServerError
                    })?;

                to_stations
                    .into_iter()
                    .filter_map(|station| station.get_id().map(|to_id| (from_id, to_id)))
                    .collect()
            }

            // 情况4: 城市 -> 站点
            (None, Some(to_station), Some(from_city), None) => {
                let to_id = self
                    .station_service
                    .get_station_by_name(to_station.trim().to_string())
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by name: {:?}", e);

                        GeneralError::InternalServerError
                    })?
                    .ok_or(TrainQueryServiceError::InvalidStationId)?
                    .get_id()
                    .expect("Station should have an ID");

                let from_stations = self
                    .station_service
                    .get_station_by_city_name(from_city)
                    .await
                    .map_err(|e| {
                        error!("Failed to get station by name: {:?}", e);

                        GeneralError::InternalServerError
                    })?;

                from_stations
                    .into_iter()
                    .filter_map(|station| station.get_id().map(|from_id| (from_id, to_id)))
                    .collect()
            }

            _ => {
                return Err(Box::new(TrainQueryServiceError::InconsistentQuery)
                    as Box<dyn ApplicationError>);
            }
        };

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
}

impl<T, U, V, W> TrainQueryServiceImpl<T, U, V, W>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
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
