use crate::Verified;
use crate::domain::model::route::{Route, RouteId};
use crate::domain::model::station::StationId;
use crate::domain::model::train::{TrainId, TrainNumber};
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::route::{RouteGraph, RouteService};
use crate::domain::service::train_schedule::{TrainScheduleService, TrainScheduleServiceError};
use crate::domain::{Identifiable, RepositoryError};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{FixedOffset, NaiveDate, TimeDelta};
use sea_orm::prelude::DateTimeWithTimeZone;
use shared::utils::TimeMeter;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

// Step 1: Define generics parameter over `RouteService` service
// Exercise 1.2.1D - 3: Your code here. (1 / 6)
pub struct TrainScheduleServiceImpl<RS, TR, TSR, RR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
    RR: RouteRepository,
{
    // Step 2: Add struct filed to store an implementation of `RouteService` service
    // Exercise 1.2.1D - 3: Your code here. (2 / 6)
    route_service: Arc<RS>,
    train_repository: Arc<TR>,
    train_schedule_repository: Arc<TSR>,
    route_repository: Arc<RR>,
    tz_offset_hour: i32,
}

impl<RS, TR, TSR, RR> TrainScheduleServiceImpl<RS, TR, TSR, RR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
    RR: RouteRepository,
{
    pub fn new(
        route_service: Arc<RS>,
        train_repository: Arc<TR>,
        train_schedule_repository: Arc<TSR>,
        route_repository: Arc<RR>,
        tz_offset_hour: i32,
    ) -> Self {
        Self {
            route_service,
            train_repository,
            train_schedule_repository,
            route_repository,
            tz_offset_hour,
        }
    }
}

//--------------------------------------------------------//
//  Internal flat connection
//--------------------------------------------------------//

struct Connection {
    departure_station: StationId,
    departure_time: u32, // 绝对秒（允许 > 86400）
    arrival_station: StationId,
    arrival_time: u32,
    train_schedule_id: TrainScheduleId,
}

fn build_connections(
    train_schedules: &[TrainSchedule],
    route_map_by_id: &HashMap<RouteId, Route>,
) -> Vec<Connection> {
    let mut connections = Vec::new();

    for schedule in train_schedules {
        let origin_offset = schedule.origin_departure_time() as u32;

        if let Some(route) = route_map_by_id.get(&schedule.route_id()) {
            for (i, from_stop) in route.stops().iter().enumerate() {
                for to_stop in route.stops().iter().skip(i + 1) {
                    connections.push(Connection {
                        departure_station: from_stop.station_id(),
                        departure_time: origin_offset + from_stop.departure_time(),
                        arrival_station: to_stop.station_id(),
                        arrival_time: origin_offset + to_stop.arrival_time(),
                        train_schedule_id: schedule.get_id().unwrap(),
                    });
                }
            }
        }
    }
    connections.sort_by_key(|c| c.departure_time);
    connections
}

fn build_station_index(graph: &RouteGraph) -> HashMap<StationId, usize> {
    graph
        .node_indices()
        .enumerate()
        .map(|(index, node)| (graph[node], index))
        .collect()
}

/*--------------------------------------------------------*
|                    辅助索引：按站点分组的出发连接         |
*--------------------------------------------------------*/
/// 为每个车站建立其所有出发 Connection 的 **索引列表**（按出发时间已排序）
fn build_outgoing_index(connections: &[Connection]) -> HashMap<StationId, Vec<usize>> {
    let mut map: HashMap<StationId, Vec<usize>> = HashMap::new();

    for (idx, conn) in connections.iter().enumerate() {
        map.entry(conn.departure_station).or_default().push(idx);
    }

    // `build_connections` 已保证全局时间排序，因此同站点内也近似有序；
    // 为严谨起见再按时间排序一次。
    for vec in map.values_mut() {
        vec.sort_by_key(|&i| connections[i].departure_time);
    }

    map
}

const MIN_TRANSFER_SEC: u32 = 10 * 60; // ≥10 分钟
const MAX_TRANSFER_SEC: u32 = 3 * 60 * 60; // ≤3 小时

impl<RS, TR, TSR, RR> TrainScheduleServiceImpl<RS, TR, TSR, RR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
    RR: RouteRepository,
{
    #[instrument(skip(self))]
    async fn load_daily_context(
        &self,
        date: NaiveDate,
    ) -> Result<
        (
            Vec<TrainSchedule>,
            HashMap<RouteId, Route>,
            Vec<Connection>,
            RouteGraph,
            HashMap<StationId, usize>,
        ),
        TrainScheduleServiceError,
    > {
        let mut meter = TimeMeter::new("load_daily_context");

        let schedules = self.get_schedules(date).await?;

        meter.meter("get schedules");

        let routes = self.route_service.get_routes().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;

        meter.meter("get routes");

        let mut route_map = HashMap::with_capacity(routes.len());
        for r in routes {
            route_map.insert(r.get_id().unwrap(), r);
        }

        let connections = build_connections(&schedules, &route_map);

        meter.meter("build connections");

        let graph = self.route_service.get_route_map().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;

        meter.meter("get route graph");

        let index_map = build_station_index(&graph);

        meter.meter("build station index");

        info!("{}", meter);

        Ok((schedules, route_map, connections, graph, index_map))
    }
}

#[async_trait]
impl<RS, TR, TSR, RR> TrainScheduleService for TrainScheduleServiceImpl<RS, TR, TSR, RR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
    RR: RouteRepository,
{
    #[instrument(skip(self))]
    async fn add_schedule(
        &self,
        train_id: TrainId,
        date: NaiveDate,
    ) -> Result<(), TrainScheduleServiceError> {
        let train = self
            .train_repository
            .find(train_id)
            .await
            .inspect_err(|e| error!("Failed to load train for id {}: {}", train_id, e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?
            .ok_or(TrainScheduleServiceError::InvalidTrainId(train_id))?;

        let mut train_schedule = TrainSchedule::new(
            None,
            train_id,
            date,
            train.default_origin_departure_time(),
            train.default_route_id(),
        );

        self.train_schedule_repository
            .save(&mut train_schedule)
            .await
            .inspect_err(|e| error!("Failed to save train schedule: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_schedules(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        self.train_schedule_repository
            .find_by_date(date)
            .await
            .inspect_err(|e| error!("Failed to load train schedule for date {}: {}", date, e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })
    }

    #[instrument(skip(self))]
    async fn get_schedule_by_train_number_and_date(
        &self,
        train_number: String,
        departure_date: NaiveDate,
    ) -> Result<Option<TrainSchedule>, TrainScheduleServiceError> {
        let train = self
            .train_repository
            .find_by_train_number(TrainNumber::from_unchecked(train_number.clone()))
            .await
            .inspect_err(|e| error!("Failed to get train: {}", e))
            .map_err(|e| match e {
                RepositoryError::InconsistentState(_) => {
                    TrainScheduleServiceError::InvalidTrainNumber(train_number)
                }
                x => {
                    TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(x))
                }
            })?;

        self.train_schedule_repository
            .find_by_id_and_date(
                train.get_id().expect("train should have id"),
                departure_date,
            )
            .await
            .inspect_err(|e| error!("Failed to get train schedule: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })
    }

    #[instrument(skip(self))]
    async fn auto_plan_schedule(
        &self,
        begin_date: NaiveDate,
        days: i32,
    ) -> Result<(), TrainScheduleServiceError> {
        info!("Auto plan schedule begin");

        let latest_date = self
            .train_schedule_repository
            .get_latest_schedule_date()
            .await
            .inspect_err(|e| error!("Failed to get latest schedule: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?;

        let latest_date = if let Some(latest_date) = latest_date {
            if latest_date > begin_date {
                latest_date
            } else {
                begin_date
            }
        } else {
            begin_date
        };

        info!("current latest schedule date: {}", latest_date);

        let days = days - ((latest_date - begin_date).num_days() as i32);

        info!("days to update: {}", days);

        if days <= 0 {
            info!("No new schedules to add, exiting auto plan.");
            return Ok(());
        }

        let trains = self
            .train_repository
            .get_trains()
            .await
            .inspect_err(|e| error!("Failed to load trains: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?;

        info!("trains to schedule: {}", trains.len());

        let mut schedule_list = Vec::with_capacity(trains.len() * (days as usize));

        for day in 0..days {
            let current_date = latest_date + chrono::Duration::days(day as i64);

            for train in &trains {
                let train_schedule = TrainSchedule::new(
                    None,
                    train.get_id().expect("train should have id"),
                    current_date,
                    train.default_origin_departure_time(),
                    train.default_route_id(),
                );

                schedule_list.push(train_schedule);
            }
        }

        info!("total schedules to save: {}", schedule_list.len());

        self.train_schedule_repository
            .save_many_no_conflict(schedule_list)
            .await
            .inspect_err(|e| error!("Failed to save train schedules: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?;

        info!("schedules saved");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn auto_plan_schedule_daemon(&self, days: i32) {
        let now = chrono::Local::now();

        let now_date = now.naive_local().date();

        info!("now date: {}, auto plan for days: {}", now_date, days);

        if let Err(e) = self.auto_plan_schedule(now_date, days).await {
            error!("Auto plan schedule failed: {}", e);
        }

        let mut interval = tokio::time::interval(chrono::Duration::days(1).to_std().unwrap());

        interval.tick().await; // 第一次tick将立即返回

        loop {
            interval.tick().await;

            let now_date = now.naive_local().date();

            info!("now date: {}, auto plan for days: 1", now_date);

            if let Err(e) = self.auto_plan_schedule(now_date, 1).await {
                error!("Auto plan schedule failed: {}", e);
            }
        }
    }

    // async fn find_schedules(
    //     &self,
    //     date: NaiveDate,
    //     from_station: StationId,
    //     to_station: StationId,
    // ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
    //     // Step 3: Load route map using `get_route_map` from stored `RouteService` implementation
    //     // Exercise 1.2.1D - 3: Your code here. (3 / 6)
    //     let route_map = self.route_service.get_route_map().await.map_err(|e| {
    //         TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
    //             e.into(),
    //         ))
    //     })?;

    //     // Step 4: Check if `from_station` and `to_station` are valid
    //     // if not, return Err(TrainScheduleServiceError::InvalidStationId)
    //     // HINT: you can check if `from_station` and `to_station` present in your RouteMap

    //     // Exercise 1.2.1D - 3: Your code here. (4 / 6)
    //     if !route_map
    //         .node_indices()
    //         .any(|node| route_map[node] == from_station)
    //         || !route_map
    //             .node_indices()
    //             .any(|node| route_map[node] == to_station)
    //     {
    //         return Err(TrainScheduleServiceError::InvalidStationId(
    //             from_station.into(),
    //         ));
    //     }

    //     // Step 5: Find k-shortest path from `from_station` to `to_station`
    //     // HINT: you may refer to algorithm in https://docs.rs/petgraph/latest/petgraph/algo/index.html

    //     // Exercise 1.2.1D - 3: Your code here. (5 / 6)

    //     let today_schedules = self.get_schedules(date).await?;

    //     let all_routes = self.route_service.get_routes().await.map_err(|e| {
    //         TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
    //             e.into(),
    //         ))
    //     })?;
    //     let mut route_lookup: HashMap<RouteId, Route> = HashMap::new();
    //     for r in all_routes {
    //         route_lookup.insert(r.get_id().unwrap(), r);
    //     }

    //     let mut result = Vec::new();

    //     for schedule in today_schedules {
    //         let route_id = schedule.route_id();
    //         if let Some(route) = route_lookup.get(&route_id) {
    //             let mut from_index: Option<usize> = None;
    //             let mut to_index: Option<usize> = None;

    //             for (i, (first_stop, second_stop)) in route.stop_pairs().enumerate() {
    //                 if first_stop.station_id() == from_station {
    //                     from_index = Some(i);
    //                 }
    //                 if second_stop.station_id() == to_station {
    //                     to_index = Some(i);
    //                 }
    //             }

    //             if let (Some(f), Some(t)) = (from_index, to_index) {
    //                 if f < t {
    //                     result.push(schedule);
    //                 }
    //             }
    //         }
    //     }

    //     // Step 6: Filter output to contain only valid schedules for specified `date`
    //     // HINT: you can get valid schedules for specified `date` using `self.get_schedules`
    //     // Exercise 1.2.1D - 3: Your code here. (6 / 6)
    //     // Good! Next, define your `TrainQueryService` application service in `base::application::service::train_query`

    //     Ok(result)
    // }

    #[instrument(skip(self, pairs))]
    async fn direct_schedules(
        &self,
        date: NaiveDate,
        pairs: &[(StationId, StationId)],
    ) -> Result<Vec<(TrainSchedule, StationId, StationId)>, TrainScheduleServiceError> {
        let mut meter = TimeMeter::new("direct_schedules");

        let yesterday = date.checked_sub_days(chrono::Days::new(1)).unwrap_or(date);

        let schedules_today = self.get_schedules(date).await?;
        let schedules_yesterday = self.get_schedules(yesterday).await?;

        let mut schedules = Vec::with_capacity(schedules_today.len() + schedules_yesterday.len());
        schedules.extend(schedules_today);
        schedules.extend(schedules_yesterday);

        meter.meter("get schedules");

        let route_map_by_id = self
            .route_service
            .get_routes()
            .await
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    e.into(),
                ))
            })?
            .into_iter()
            .map(|r| (r.get_id().unwrap(), r))
            .collect::<HashMap<_, _>>();

        meter.meter("get routes");

        let want: HashSet<_> = pairs.iter().copied().collect();

        let mut result = Vec::new();

        let mut route_pos_cache: HashMap<RouteId, HashMap<StationId, usize>> =
            HashMap::with_capacity(route_map_by_id.len());

        for schedule in &schedules {
            let route_id = schedule.route_id();

            let route = match route_map_by_id.get(&route_id) {
                Some(r) => r,
                None => continue,
            };

            let pos_map: HashMap<StationId, usize> = match route_pos_cache.entry(route_id) {
                Entry::Occupied(o) => o.get().clone(),
                Entry::Vacant(v) => {
                    let mut m = HashMap::with_capacity(route.stops().len());
                    for (idx, stop) in route.stops().iter().enumerate() {
                        m.insert(stop.station_id(), idx);
                    }
                    v.insert(m).clone()
                }
            };

            for &(from, to) in &want {
                if let (Some(&i_from), Some(&i_to)) = (pos_map.get(&from), pos_map.get(&to)) {
                    if i_from < i_to {
                        let stop = route.stops()[i_from];
                        let departure_offset =
                            stop.departure_time() + schedule.origin_departure_time() as u32;
                        let departure_datetime = schedule.date().and_hms_opt(0, 0, 0).unwrap()
                            + chrono::Duration::seconds(departure_offset as i64);

                        if departure_datetime.date() == date {
                            result.push((schedule.clone(), from, to));
                            break;
                        }
                    }
                }
            }
        }

        result.sort_by_key(|(s, _, _)| s.origin_departure_time());

        meter.meter("calc");

        info!("{}", meter);

        Ok(result)
    }

    //--------------------------------------------------------//
    //  换乘查询 (k = 1) —— 列举 **所有** 满足条件的方案
    //--------------------------------------------------------//
    /// 返回值含义：`(两段列车ID, Some(中转站))`
    ///
    /// * **只允许一次换乘**（两段列车，且必须在同一方向继续前进）。
    /// * 对于同一出发/到达车站对，**列举所有可行方案**，并去重。
    /// * 为保证性能：
    ///   1. 预先对 `Connection` 构建 **出发索引**（O(M) 内存，M≈连接数）。
    ///   2. 遍历时只访问与给定站点相关的连接，避免 O(N·M) 的暴力交叉。
    #[instrument(skip(self, pairs))]
    async fn transfer_schedules(
        &self,
        date: chrono::NaiveDate,
        pairs: &[(StationId, StationId)],
    ) -> Result<
        Vec<(
            Vec<TrainScheduleId>,
            StationId,
            StationId,
            Option<StationId>,
        )>,
        TrainScheduleServiceError,
    > {
        let mut meter = TimeMeter::new("transfer_schedules");

        let yesterday = date.checked_sub_days(chrono::Days::new(1)).unwrap_or(date);

        let schedules_today = self.get_schedules(date).await?;
        let schedules_yesterday = self.get_schedules(yesterday).await?;

        let mut schedules = Vec::with_capacity(schedules_today.len() + schedules_yesterday.len());
        schedules.extend(schedules_today);
        schedules.extend(schedules_yesterday);

        meter.meter("get schedules");

        let (_, _, connections, _graph, _index_map) = self.load_daily_context(date).await?;

        meter.meter("load daily context");

        // 站点 -> 出发 Connection 索引表
        let outgoing_index = build_outgoing_index(&connections);

        meter.meter("build outgoing index");

        let schedule_map: HashMap<TrainScheduleId, &TrainSchedule> =
            schedules.iter().map(|s| (s.get_id().unwrap(), s)).collect();

        let route_map_by_id = self
            .route_service
            .get_routes()
            .await
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    e.into(),
                ))
            })?
            .into_iter()
            .map(|r| (r.get_id().unwrap(), r))
            .collect::<HashMap<_, _>>();

        let mut route_pos_cache: HashMap<RouteId, HashMap<StationId, usize>> =
            HashMap::with_capacity(route_map_by_id.len());

        let mut all_solutions = Vec::<(
            Vec<TrainScheduleId>,
            StationId,
            StationId,
            Option<StationId>,
        )>::new();

        for &(origin, dest) in pairs {
            let Some(first_leg_indices) = outgoing_index.get(&origin) else {
                continue;
            };

            // 用于去重，避免同方案多次加入
            let mut seen: HashSet<(TrainScheduleId, TrainScheduleId, StationId)> = HashSet::new();

            for &i in first_leg_indices {
                let first = &connections[i];

                let first_schedule = match schedule_map.get(&first.train_schedule_id) {
                    Some(s) => *s,
                    None => continue,
                };

                let first_route = match route_map_by_id.get(&first_schedule.route_id()) {
                    Some(r) => r,
                    None => continue,
                };

                let pos_map = match route_pos_cache.entry(first_schedule.route_id()) {
                    Entry::Occupied(o) => o.get().clone(),
                    Entry::Vacant(v) => {
                        let mut m = HashMap::with_capacity(first_route.stops().len());
                        for (idx, stop) in first_route.stops().iter().enumerate() {
                            m.insert(stop.station_id(), idx);
                        }
                        v.insert(m).clone()
                    }
                };

                let &origin_idx = match pos_map.get(&origin) {
                    Some(idx) => idx,
                    None => continue,
                };

                let stop = first_route.stops()[origin_idx];
                let departure_offset =
                    stop.departure_time() + first_schedule.origin_departure_time() as u32;
                let departure_datetime = first_schedule.date().and_hms_opt(0, 0, 0).unwrap()
                    + chrono::Duration::seconds(departure_offset as i64);

                if departure_datetime.date() != date {
                    continue;
                }

                let earliest_next_dep = first.arrival_time + MIN_TRANSFER_SEC;
                let latest_next_dep = first.arrival_time + MAX_TRANSFER_SEC;

                let Some(second_leg_indices) = outgoing_index.get(&first.arrival_station) else {
                    continue;
                };

                // ---------- 二分定位可行第二段起始位置 ---------- //
                let start_idx = match second_leg_indices
                    .binary_search_by_key(&earliest_next_dep, |&idx| {
                        connections[idx].departure_time
                    }) {
                    Ok(pos) | Err(pos) => pos,
                };

                for &j in &second_leg_indices[start_idx..] {
                    let second = &connections[j];

                    if second.departure_time > latest_next_dep {
                        break;
                    }

                    if second.departure_station != first.arrival_station {
                        continue;
                    }

                    if second.arrival_station != dest {
                        continue;
                    }

                    if first.train_schedule_id == second.train_schedule_id {
                        continue;
                    }

                    let mid_station = first.arrival_station;
                    let key = (
                        first.train_schedule_id,
                        second.train_schedule_id,
                        mid_station,
                    );

                    if seen.insert(key) {
                        all_solutions.push((
                            vec![first.train_schedule_id, second.train_schedule_id],
                            origin,
                            dest,
                            Some(mid_station),
                        ));
                    }
                }
            }
        }

        meter.meter("calc");

        info!("{}", meter);

        Ok(all_solutions)
    }

    async fn get_station_arrival_time(
        &self,
        train_schedule_id: TrainScheduleId,
        station_id: StationId,
    ) -> Result<sea_orm::prelude::DateTimeWithTimeZone, TrainScheduleServiceError> {
        let today = chrono::Local::now().date_naive();
        let mut found_schedule = None;

        for days_diff in -30..=30 {
            let check_date = today + chrono::Duration::days(days_diff);
            let schedules = self.get_schedules(check_date).await?;

            if let Some(schedule) = schedules
                .iter()
                .find(|s| s.get_id() == Some(train_schedule_id))
            {
                found_schedule = Some(schedule.clone());
                break;
            }
        }

        let train_schedule = found_schedule.ok_or_else(|| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                anyhow::anyhow!("Train schedule not found"),
            ))
        })?;

        let all_routes = self.route_service.get_routes().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;

        let route = all_routes
            .into_iter()
            .find(|r| r.get_id() == Some(train_schedule.route_id()))
            .ok_or_else(|| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    anyhow::anyhow!("Route not found for train schedule"),
                ))
            })?;

        let stop = route
            .stops()
            .iter()
            .find(|stop| stop.station_id() == station_id)
            .ok_or_else(|| TrainScheduleServiceError::InvalidStationId(station_id.into()))?;

        let base_date = train_schedule.date();

        let origin_departure_seconds = train_schedule.origin_departure_time() as i64;

        let station_arrival_offset = stop.arrival_time() as i64;

        let base_datetime = chrono::NaiveDateTime::new(
            base_date,
            chrono::NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap(),
        );

        let arrival_datetime = base_datetime
            + chrono::Duration::seconds(origin_departure_seconds)
            + chrono::Duration::seconds(station_arrival_offset);

        let arrival_time = arrival_datetime
            .and_local_timezone(FixedOffset::east_opt(self.tz_offset_hour * 3600).unwrap())
            .unwrap();

        Ok(arrival_time)
    }

    #[instrument(skip(self))]
    async fn get_terminal_arrival_time(
        &self,
        train_number: TrainNumber<Verified>,
        origin_departure_time: DateTimeWithTimeZone,
    ) -> Result<DateTimeWithTimeZone, TrainScheduleServiceError> {
        let train = self
            .train_repository
            .find_by_train_number(train_number.clone())
            .await
            .inspect_err(|e| error!("Failed to get train for verified train number: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?;

        let train_id = train.get_id().expect("train should have id");
        let origin_departure_date = origin_departure_time.date_naive();

        let train_schedule = self
            .train_schedule_repository
            .find_by_id_and_date(train_id, origin_departure_date)
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to get train schedule for train {} on date {}: {}",
                    train_id, origin_departure_date, e
                )
            })
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?
            .ok_or_else(|| {
                warn!(
                    "no train schedule found for train {} on date {}",
                    train_id, origin_departure_date
                );

                TrainScheduleServiceError::InvalidTrainNumber(train_number.to_string())
            })?;

        let route_id = train_schedule.route_id();

        let route = self
            .route_repository
            .find(route_id)
            .await
            .inspect_err(|e| error!("Failed to get route for train schedule: {}", e))
            .map_err(|e| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RepositoryError(e))
            })?
            .ok_or(TrainScheduleServiceError::InfrastructureError(
                ServiceError::RepositoryError(RepositoryError::InconsistentState(anyhow!(
                    "no route found for route id: {}",
                    route_id
                ))),
            ))?;

        let terminal_stop = route
            .stops()
            .iter()
            .max_by(|a, b| a.order().cmp(&b.order()))
            .expect("route should have at least one stop");

        let terminal_arrival_offset = terminal_stop.arrival_time() as i64;

        let arrival_time = origin_departure_time + TimeDelta::seconds(terminal_arrival_offset);

        Ok(arrival_time)
    }
}
