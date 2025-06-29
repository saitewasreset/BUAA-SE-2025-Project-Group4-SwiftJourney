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
    DirectTrainQueryDTO, SeatInfoDTO, StoppingStationInfo, TrainInfoDTO, TrainQueryResponseDTO,
    TrainQueryService, TrainQueryServiceError, TransferSolutionDTO, TransferTrainQueryDTO,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::station::StationId;
use crate::domain::model::train::Train;
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::route::RouteService;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::station::StationService;
use crate::domain::service::train_schedule::{TrainScheduleService, TrainScheduleServiceError};
use async_trait::async_trait;
use chrono::{Duration, FixedOffset, NaiveDate};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::prelude::DateTimeWithTimeZone;
use shared::utils::TimeMeter;
use std::collections::HashMap;
use tracing::{error, info, instrument};

// Thinking 1.2.1D - 4: 为何需要使用`+ 'static + Send + Sync`约束泛型参数？
// Thinking 1.2.1D - 5: 为何需要使用`Arc<T>`存储领域服务？为何无需使用`Arc<Mutex<T>>`？
pub struct TrainQueryServiceImpl<T, U, W, SMS, RR, TR, SR>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
    RR: RouteRepository,
    TR: TrainRepository,
    SR: StationRepository,
{
    // Step 3: Store service instance you need using `Arc<T>` and generics parameter
    // HINT: You may refer to `UserManagerServiceImpl` for example
    // Exercise 1.2.1D - 5: Your code here. (2 / 6)
    train_schedule_service: Arc<T>,
    station_service: Arc<U>,
    route_service: Arc<W>,
    session_manager_service: Arc<SMS>,
    route_repository: Arc<RR>,
    train_repository: Arc<TR>,
    station_repository: Arc<SR>,
    tz_offset_hour: i32,
}

// Step 4: Implement `new` associate function for `TrainQueryServiceImpl`
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (3 / 6)
impl<T, U, W, SMS, RR, TR, SR> TrainQueryServiceImpl<T, U, W, SMS, RR, TR, SR>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
    RR: RouteRepository,
    TR: TrainRepository,
    SR: StationRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        train_schedule_service: Arc<T>,
        station_service: Arc<U>,
        route_service: Arc<W>,
        session_manager_service: Arc<SMS>,
        route_repository: Arc<RR>,
        train_repository: Arc<TR>,
        station_repository: Arc<SR>,
        tz_offset_hour: i32,
    ) -> Self {
        TrainQueryServiceImpl {
            train_schedule_service,
            station_service,
            route_service,
            session_manager_service,
            route_repository,
            train_repository,
            station_repository,
            tz_offset_hour,
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
                        TrainQueryServiceError::InvalidCityId
                    })?;
                Ok(stations.into_iter().filter_map(|st| st.get_id()).collect())
            }
            _ => Err(Box::new(TrainQueryServiceError::InconsistentQuery)),
        }
    }

    async fn verify_session(&self, session_id: &str) -> Result<(), Box<dyn ApplicationError>> {
        if !self
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
impl<T, U, W, SMS, RR, TR, SR> TrainQueryService for TrainQueryServiceImpl<T, U, W, SMS, RR, TR, SR>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
    RR: RouteRepository,
    TR: TrainRepository,
    SR: StationRepository,
{
    #[instrument(skip(self))]
    async fn query_train(
        &self,
        cmd: TrainScheduleQueryCommand,
    ) -> Result<TrainQueryResponseDTO, Box<dyn ApplicationError>> {
        self.verify_session(cmd.session_id.as_str()).await?;

        let departure_date =
            NaiveDate::parse_from_str(&cmd.departure_date, "%Y-%m-%d").map_err(|_| {
                GeneralError::BadRequest(format!("Invalid date: {}", cmd.departure_date))
            })?;

        let schedule = self
            .train_schedule_service
            .get_schedule_by_train_number_and_date(cmd.train_number.clone(), departure_date)
            .await
            .map_err(|e| match e {
                TrainScheduleServiceError::InvalidTrainNumber(_for_super_earth) => {
                    GeneralError::NotFound
                }
                x => {
                    error!("Failed to get train schedule: {}", x);

                    GeneralError::InternalServerError
                }
            })?
            .ok_or(GeneralError::BadRequest(format!(
                "invalid train: {} {}",
                cmd.train_number, cmd.departure_date
            )))?;

        let route = self
            .route_repository
            .get_by_train_schedule(schedule.get_id().expect("Train schedule should have id"))
            .await
            .inspect_err(|e| error!("Failed to get route by train schedule id: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or_else(|| {
                error!(
                    "Inconsistent: No route found for train schedule id: {:?}",
                    schedule.get_id()
                );
                GeneralError::InternalServerError
            })?;

        let station = self
            .station_service
            .get_stations()
            .await
            .inspect_err(|e| error!("Failed to get stations: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        let station_id_to_name = station
            .into_iter()
            .map(|s| {
                (
                    s.get_id().expect("Station should have id"),
                    s.name().to_string(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut stopping_station_list = Vec::with_capacity(route.stops().len());

        let mut origin_station = None;
        let mut origin_departure_time = None;
        let mut origin_departure_date = None;
        let mut terminal_station = None;
        let mut terminal_arrival_time = None;

        for stop in route.stops() {
            let station_name = station_id_to_name
                .get(&stop.station_id())
                .ok_or_else(|| {
                    error!(
                        "Inconsistent: No station found for id: {}",
                        stop.station_id()
                    );

                    GeneralError::InternalServerError
                })?
                .clone();

            let arrival_time_secs = stop.arrival_time() + schedule.origin_departure_time() as u32;
            let departure_time_secs =
                stop.departure_time() + schedule.origin_departure_time() as u32;

            if stop.order() == 0 {
                origin_station = Some(station_name.clone());

                let departure_datetime = departure_date
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .checked_add_signed(Duration::seconds(arrival_time_secs as i64))
                    .unwrap()
                    .and_local_timezone(FixedOffset::east_opt(self.tz_offset_hour * 3600).unwrap())
                    .unwrap();

                origin_departure_time = Some(departure_datetime.to_rfc3339());

                origin_departure_date = Some(departure_datetime.date_naive().to_string());
            } else if stop.order() == (route.stops().len() - 1) as u32 {
                terminal_station = Some(station_name.clone());

                terminal_arrival_time = Some(
                    departure_date
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .checked_add_signed(Duration::seconds(arrival_time_secs as i64))
                        .unwrap()
                        .and_local_timezone(
                            FixedOffset::east_opt(self.tz_offset_hour * 3600).unwrap(),
                        )
                        .unwrap()
                        .to_rfc3339(),
                );
            }

            let arrival_time_opt = if stop.order() == 0 {
                None
            } else {
                Some(
                    departure_date
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .checked_add_signed(Duration::seconds(arrival_time_secs as i64))
                        .unwrap()
                        .and_local_timezone(
                            FixedOffset::east_opt(self.tz_offset_hour * 3600).unwrap(),
                        )
                        .unwrap()
                        .to_rfc3339(),
                )
            };

            let departure_time_opt = if stop.order() == (route.stops().len() - 1) as u32 {
                None
            } else {
                Some(
                    departure_date
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .checked_add_signed(Duration::seconds(departure_time_secs as i64))
                        .unwrap()
                        .and_local_timezone(
                            FixedOffset::east_opt(self.tz_offset_hour * 3600).unwrap(),
                        )
                        .unwrap()
                        .to_rfc3339(),
                )
            };

            stopping_station_list.push(StoppingStationInfo {
                station_name,
                arrival_time: arrival_time_opt,
                departure_time: departure_time_opt,
            });
        }

        let origin_station = origin_station.expect("should have origin station");
        let origin_departure_time =
            origin_departure_time.expect("should have origin departure time");
        let origin_departure_date =
            origin_departure_date.expect("should have origin departure date");
        let terminal_station = terminal_station.expect("should have terminal station");
        let terminal_arrival_time =
            terminal_arrival_time.expect("should have terminal arrival time");

        Ok(TrainQueryResponseDTO {
            origin_station,
            origin_departure_time,
            departure_date: origin_departure_date,
            terminal_station,
            terminal_arrival_time,
            route: stopping_station_list,
        })
    }
    #[instrument(skip(self))]
    async fn query_direct_trains(
        &self,
        cmd: DirectTrainQueryCommand,
    ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>> {
        let mut meter = TimeMeter::new("DirectTrainQuery");

        self.verify_session(cmd.session_id.as_str()).await?;

        cmd.validate()?;

        meter.meter("verify session and command");

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

        meter.meter("resolve station ids");

        let schedules = self
            .train_schedule_service
            .direct_schedules(cmd.departure_time, &station_pairs)
            .await
            .map_err(|e| {
                error!("Failed to get direct schedules: {:?}", e);
                GeneralError::InternalServerError
            })?;

        meter.meter("get direct schedules");

        let routes = self.route_service.get_routes().await.map_err(|e| {
            error!("Failed to get routes: {:?}", e);

            GeneralError::InternalServerError
        })?;

        meter.meter("get routes");

        let mut infos = Vec::new();

        let station_list = self
            .station_repository
            .load()
            .await
            .inspect_err(|e| error!("failed to load stations: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        meter.meter("load stations");

        let station_id_to_name = station_list
            .into_iter()
            .map(|s| {
                (
                    s.get_id().expect("Station should have id"),
                    s.name().to_string(),
                )
            })
            .collect::<HashMap<_, _>>();

        let train_list = self
            .train_repository
            .get_trains()
            .await
            .inspect_err(|_for_super_earth| error!("Failed to load trains"))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        meter.meter("load trains");

        let train_id_to_train = train_list
            .into_iter()
            .map(|t| (t.get_id().expect("Train should have id"), t))
            .collect::<HashMap<_, _>>();

        for (sch, from_station, to_station) in schedules {
            let train = train_id_to_train
                .get(&sch.train_id())
                .cloned()
                .ok_or_else(|| {
                    error!(
                        "Inconsistent: No train found for schedule id: {}",
                        sch.get_id().unwrap()
                    );
                    GeneralError::InternalServerError
                })?;

            infos.push(
                self.build_dto(
                    &sch,
                    train,
                    &routes,
                    &station_id_to_name,
                    cmd.departure_time,
                    Some(from_station),
                    Some(to_station),
                )
                .await?,
            );
        }

        meter.meter("build train info DTOs");

        // infos.sort_by(|a, b| {
        //     let a_time = DateTimeWithTimeZone::parse_from_rfc3339(&a.departure_time).unwrap();
        //     let b_time = DateTimeWithTimeZone::parse_from_rfc3339(&b.departure_time).unwrap();
        //     a_time.cmp(&b_time)
        // });

        // // 限制结果数量为50个，应前端要求
        // if infos.len() > 50 {
        //     infos.truncate(50);
        // }

        // meter.meter("sort and limit results");

        info!("{}", meter);

        Ok(DirectTrainQueryDTO { solutions: infos })
    }

    #[instrument(skip(self))]
    async fn query_transfer_trains(
        &self,
        cmd: TransferTrainQueryCommand,
    ) -> Result<TransferTrainQueryDTO, Box<dyn ApplicationError>> {
        let mut meter = TimeMeter::new("TransferTrainQuery");

        self.verify_session(cmd.session_id.as_str()).await?;
        cmd.validate()?;

        meter.meter("verify session and command");

        let from_ids = self
            .resolve_station_ids(&cmd.departure_station, &cmd.departure_city)
            .await?;
        let to_ids = self
            .resolve_station_ids(&cmd.arrival_station, &cmd.arrival_city)
            .await?;

        meter.meter("resolve station ids");

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

        meter.meter("get transfer schedules");

        let routes = self.route_service.get_routes().await.map_err(|e| {
            error!("Failed to get routes: {:?}", e);
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        meter.meter("get routes");

        let schedules = self
            .train_schedule_service
            .get_schedules(cmd.departure_time)
            .await
            .map_err(|e| {
                error!("Failed to get schedules: {:?}", e);
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        meter.meter("get schedules");

        let schedule_by_id: HashMap<_, _> = schedules
            .iter()
            .filter_map(|s| s.get_id().map(|id| (id, s)))
            .collect();

        let station_list = self
            .station_repository
            .load()
            .await
            .inspect_err(|e| error!("failed to load stations: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        meter.meter("load stations");

        let train_list = self
            .train_repository
            .get_trains()
            .await
            .inspect_err(|_for_super_earth| error!("Failed to load trains"))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        meter.meter("load trains");

        let train_id_to_train = train_list
            .into_iter()
            .map(|t| (t.get_id().expect("Train should have id"), t))
            .collect::<HashMap<_, _>>();

        let station_id_to_name = station_list
            .into_iter()
            .map(|s| {
                (
                    s.get_id().expect("Station should have id"),
                    s.name().to_string(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut solutions = Vec::new();
        for (schedule_ids, from_station, to_station, mid_station_opt) in transfer_solutions {
            if schedule_ids.len() != 2 || mid_station_opt.is_none() {
                continue;
            }

            let mid_station = mid_station_opt.unwrap();

            let station_name = station_id_to_name
                .get(&mid_station)
                .cloned()
                .ok_or_else(|| {
                    error!("Inconsistent: No station found for id: {}", mid_station);

                    GeneralError::InternalServerError
                })?;

            let first_schedule = match schedule_by_id.get(&schedule_ids[0]) {
                Some(s) => s,
                None => continue,
            };

            let second_schedule = match schedule_by_id.get(&schedule_ids[1]) {
                Some(s) => s,
                None => continue,
            };

            let first_train = match train_id_to_train.get(&first_schedule.train_id()).cloned() {
                Some(t) => t,
                None => continue,
            };

            let mut first_dto = match self
                .build_dto(
                    first_schedule,
                    first_train,
                    &routes,
                    &station_id_to_name,
                    cmd.departure_time,
                    Some(from_station),
                    Some(mid_station),
                )
                .await
            {
                Ok(dto) => dto,
                Err(_) => continue,
            };

            let second_train = match train_id_to_train.get(&second_schedule.train_id()).cloned() {
                Some(t) => t,
                None => continue,
            };

            let mut second_dto = match self
                .build_dto(
                    second_schedule,
                    second_train,
                    &routes,
                    &station_id_to_name,
                    cmd.departure_time,
                    Some(mid_station),
                    Some(to_station),
                )
                .await
            {
                Ok(dto) => dto,
                Err(_) => continue,
            };

            let first_mid_idx = match first_dto
                .route
                .iter()
                .position(|stop| stop.station_name == station_name)
            {
                Some(idx) => idx,
                None => continue,
            };

            first_dto.arrival_station = station_name.clone();
            first_dto.arrival_time = first_dto.route[first_mid_idx]
                .arrival_time
                .clone()
                .expect("missing arrival time");

            // first_dto.route.truncate(first_mid_idx + 1);

            let first_dep_dt =
                DateTimeWithTimeZone::parse_from_rfc3339(&first_dto.departure_time).unwrap();

            let first_arr_dt =
                DateTimeWithTimeZone::parse_from_rfc3339(&first_dto.arrival_time).unwrap();

            first_dto.travel_time = (first_arr_dt - first_dep_dt).num_seconds() as u32;

            let second_mid_idx = match second_dto
                .route
                .iter()
                .position(|stop| stop.station_name == station_name)
            {
                Some(idx) => idx,
                None => continue,
            };

            second_dto.departure_station = station_name.clone();
            second_dto.departure_time = second_dto.route[second_mid_idx]
                .departure_time
                .clone()
                .expect("missing departure time");

            // second_dto.route = second_dto.route[second_mid_idx..].to_vec();

            let second_dep_dt =
                DateTimeWithTimeZone::parse_from_rfc3339(&second_dto.departure_time).unwrap();
            let second_arr_dt =
                DateTimeWithTimeZone::parse_from_rfc3339(&second_dto.arrival_time).unwrap();

            second_dto.travel_time = (second_arr_dt - second_dep_dt).num_seconds() as u32;

            let relaxing_time = if second_dep_dt > first_arr_dt {
                (second_dep_dt - first_arr_dt).num_seconds() as u32
            } else {
                (second_dep_dt + Duration::days(1) - first_arr_dt).num_seconds() as u32
            };

            solutions.push(TransferSolutionDTO {
                first_ride: first_dto,
                second_ride: second_dto,
                relaxing_time,
            });
        }

        meter.meter("build transfer solutions");

        solutions.sort_by(|a, b| {
            let a_total_time =
                a.first_ride.travel_time + a.relaxing_time + a.second_ride.travel_time;
            let b_total_time =
                b.first_ride.travel_time + b.relaxing_time + b.second_ride.travel_time;
            a_total_time.cmp(&b_total_time)
        });

        // 限制结果数量为50个，应前端要求
        if solutions.len() > 50 {
            solutions.truncate(50);
        }

        meter.meter("sort and limit results");

        info!("{}", meter);

        Ok(TransferTrainQueryDTO { solutions })
    }
}

impl<T, U, W, SMS, RR, TR, SR> TrainQueryServiceImpl<T, U, W, SMS, RR, TR, SR>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    SMS: SessionManagerService,
    RR: RouteRepository,
    TR: TrainRepository,
    SR: StationRepository,
{
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self, routes, station_id_to_name))]
    async fn build_dto(
        &self,
        sch: &crate::domain::model::train_schedule::TrainSchedule,
        train: Train,
        routes: &[crate::domain::model::route::Route],
        station_id_to_name: &HashMap<StationId, String>,
        date: NaiveDate,
        user_departure_station: Option<StationId>,
        user_arrival_station: Option<StationId>,
    ) -> Result<TrainInfoDTO, Box<dyn ApplicationError>> {
        // ——— 路线、停站 ———
        let route = routes
            .iter()
            .find(|r| r.get_id() == Some(sch.route_id()))
            .ok_or(TrainQueryServiceError::InvalidStationId)?;

        let mut stopping = Vec::<StoppingStationInfo>::new();
        for stop in route.stops() {
            let name = station_id_to_name
                .get(&stop.station_id())
                .ok_or_else(|| {
                    error!(
                        "Inconsistent: no station found for id {}",
                        stop.station_id()
                    );

                    GeneralError::InternalServerError
                })?
                .clone();

            let arrival_time_secs = stop.arrival_time() + sch.origin_departure_time() as u32;
            let departure_time_secs = stop.departure_time() + sch.origin_departure_time() as u32;

            let arrival_time_opt = if stop.order() == 0 {
                None
            } else {
                Some(
                    date.and_hms_opt(0, 0, 0)
                        .unwrap()
                        .checked_add_signed(Duration::seconds(arrival_time_secs as i64))
                        .unwrap()
                        .and_local_timezone(
                            FixedOffset::east_opt(self.tz_offset_hour * 3600).unwrap(),
                        )
                        .unwrap()
                        .to_rfc3339(),
                )
            };

            let departure_time_opt = if stop.order() == (route.stops().len() - 1) as u32 {
                None
            } else {
                Some(
                    date.and_hms_opt(0, 0, 0)
                        .unwrap()
                        .checked_add_signed(Duration::seconds(departure_time_secs as i64))
                        .unwrap()
                        .and_local_timezone(
                            FixedOffset::east_opt(self.tz_offset_hour * 3600).unwrap(),
                        )
                        .unwrap()
                        .to_rfc3339(),
                )
            };

            stopping.push(StoppingStationInfo {
                station_name: name,
                arrival_time: arrival_time_opt,
                departure_time: departure_time_opt,
            });
        }

        // ——— 列车 / 座位 ———
        let stations_count = stopping.len() - 1;

        let mut seat_info = HashMap::new();
        for seat in train.seats().values() {
            let unit_price = seat.unit_price().to_f64().unwrap_or(0.0) as u32;
            let total_price = unit_price * stations_count as u32;
            seat_info.insert(
                seat.name().to_string(),
                SeatInfoDTO {
                    seat_type: seat.name().to_string(),
                    left: seat.capacity(),
                    price: total_price,
                },
            );
        }

        // ——— 其余字段 ———
        let user_dep_station_name = user_departure_station
            .and_then(|id| station_id_to_name.get(&id).cloned())
            .unwrap();

        let user_arr_station_name = user_arrival_station
            .and_then(|id| station_id_to_name.get(&id).cloned())
            .unwrap();

        let user_dep_idx = stopping
            .iter()
            .position(|stop| stop.station_name == user_dep_station_name)
            .expect("departure station should exist in stopping");

        let user_arr_idx = stopping
            .iter()
            .position(|stop| stop.station_name == user_arr_station_name)
            .expect("arrival station should exist in stopping");

        let dep_time = stopping[user_dep_idx]
            .departure_time
            .as_ref()
            .expect("departure time should exist for user departure station");

        let arr_time = stopping[user_arr_idx]
            .arrival_time
            .as_ref()
            .expect("arrival time should exist for user arrival station");
        let dep_dt = DateTimeWithTimeZone::parse_from_rfc3339(dep_time).unwrap();
        let arr_dt = DateTimeWithTimeZone::parse_from_rfc3339(arr_time).unwrap();

        Ok(TrainInfoDTO {
            departure_station: user_departure_station
                .and_then(|id| station_id_to_name.get(&id).cloned())
                .unwrap(),
            departure_time: dep_time.clone(),
            arrival_station: user_arrival_station
                .and_then(|id| station_id_to_name.get(&id).cloned())
                .unwrap(),
            arrival_time: arr_time.clone(),
            origin_station: stopping.first().unwrap().station_name.clone(),
            origin_departure_time: stopping
                .first()
                .unwrap()
                .departure_time
                .clone()
                .expect("departure time should exist for origin stop"),
            terminal_station: stopping.last().unwrap().station_name.clone(),
            terminal_arrival_time: stopping
                .last()
                .unwrap()
                .arrival_time
                .clone()
                .expect("arrival time should exist for terminal stop"),
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
