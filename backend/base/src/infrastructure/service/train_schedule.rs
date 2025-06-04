use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::domain::model::route::{Route, RouteId};
use crate::domain::model::station::StationId;
use crate::domain::model::train::{TrainId, TrainNumber};
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::ServiceError;
use crate::domain::service::route::{RouteGraph, RouteService};
use crate::domain::service::train_schedule::{TrainScheduleService, TrainScheduleServiceError};
use crate::domain::{Identifiable, RepositoryError};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, Timelike};
use tracing::{error, info, instrument};

// Step 1: Define generics parameter over `RouteService` service
// Exercise 1.2.1D - 3: Your code here. (1 / 6)
pub struct TrainScheduleServiceImpl<RS, TR, TSR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
{
    // Step 2: Add struct filed to store an implementation of `RouteService` service
    // Exercise 1.2.1D - 3: Your code here. (2 / 6)
    route_service: Arc<RS>,
    train_repository: Arc<TR>,
    train_schedule_repository: Arc<TSR>,
    tz_offset_hour: i32,
}

impl<RS, TR, TSR> TrainScheduleServiceImpl<RS, TR, TSR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
{
    pub fn new(
        route_service: Arc<RS>,
        train_repository: Arc<TR>,
        train_schedule_repository: Arc<TSR>,
        tz_offset_hour: i32,
    ) -> Self {
        Self {
            route_service,
            train_repository,
            train_schedule_repository,
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
            for (prev_stop, next_stop) in route.stop_pairs() {
                connections.push(Connection {
                    departure_station: prev_stop.station_id(),
                    departure_time: origin_offset + prev_stop.departure_time(),
                    arrival_station: next_stop.station_id(),
                    arrival_time: origin_offset + next_stop.arrival_time(),
                    train_schedule_id: schedule.get_id().unwrap(),
                });
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

//--------------------------------------------------------//
//  Earliest‑arrival CSA (k = 1)
//--------------------------------------------------------//

const INF_SEC: u32 = u32::MAX / 2;
const MIN_TRANSFER_SEC: u32 = 10 * 60; // 10 min

#[derive(Clone, Copy, Default)]
struct Predecessor {
    previous_index: Option<usize>,
    previous_schedule: Option<TrainScheduleId>,
    first_schedule: Option<TrainScheduleId>, // 仅换乘层
    transfer_index: Option<usize>,           // 仅换乘层
}

#[derive(Clone, Copy, Default)]
struct Arrival {
    time: u32,
    pred: Predecessor,
}

/// CSA‑k=1。返回 (直达层, 换乘层)
fn earliest_arrival_k1(
    origin_index: usize,
    station_cnt: usize,
    origin_start_sec: u32,
    connections: &[Connection],
    index_map: &HashMap<StationId, usize>,
) -> (Vec<Arrival>, Vec<Arrival>) {
    let mut level0 = vec![
        Arrival {
            time: INF_SEC,
            ..Default::default()
        };
        station_cnt
    ];
    let mut level1 = level0.clone();

    level0[origin_index].time = origin_start_sec;
    level1[origin_index].time = origin_start_sec;

    for conn in connections {
        let dep = index_map[&conn.departure_station];
        let arr = index_map[&conn.arrival_station];

        /* ---------- 直达层 ---------- */
        if level0[dep].time <= conn.departure_time && conn.arrival_time < level0[arr].time {
            level0[arr] = Arrival {
                time: conn.arrival_time,
                pred: Predecessor {
                    previous_index: Some(dep),
                    previous_schedule: Some(conn.train_schedule_id),
                    ..Default::default()
                },
            };
            // 同步到换乘层
            if conn.arrival_time < level1[arr].time {
                level1[arr] = level0[arr];
            }
        }

        /* ---------- 换乘层 ---------- */
        let ready_time = level1[dep].time.saturating_add(MIN_TRANSFER_SEC);
        if ready_time <= conn.departure_time && conn.arrival_time < level1[arr].time {
            let dep_pred = level1[dep].pred;
            level1[arr] = Arrival {
                time: conn.arrival_time,
                pred: Predecessor {
                    previous_index: Some(dep),
                    previous_schedule: Some(conn.train_schedule_id),
                    first_schedule: dep_pred.previous_schedule.or(dep_pred.first_schedule),
                    transfer_index: Some(dep),
                },
            };
        }
    }
    (level0, level1)
}

/* -------- 回溯工具 -------- */

fn backtrack_direct(level0: &[Arrival], dst_index: usize) -> Vec<TrainScheduleId> {
    let mut ids = Vec::<TrainScheduleId>::new();
    let mut current = Some(dst_index);

    while let Some(i) = current {
        if let Some(sid) = level0[i].pred.previous_schedule {
            ids.push(sid);
        }
        current = level0[i].pred.previous_index;
    }
    ids.reverse();
    ids
}

fn backtrack_transfer(
    level1: &[Arrival],
    dst_index: usize,
    index_station_map: &HashMap<usize, StationId>,
) -> (Vec<TrainScheduleId>, Option<StationId>) {
    let mut ids = Vec::<TrainScheduleId>::new();
    let mut mid_station = None;
    let mut current = Some(dst_index);

    while let Some(i) = current {
        if let Some(sid) = level1[i].pred.previous_schedule {
            ids.push(sid);
        }

        if mid_station.is_none() {
            if let Some(mid_index) = level1[i].pred.transfer_index {
                mid_station = index_station_map.get(&mid_index).copied();
            }
        }

        current = level1[i].pred.previous_index;
    }

    if let Some(first) = level1[dst_index].pred.first_schedule {
        ids.push(first);
    }

    ids.reverse();
    ids.dedup();
    (ids, mid_station)
}

impl<RS, TR, TSR> TrainScheduleServiceImpl<RS, TR, TSR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
{
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
        let schedules = self.get_schedules(date).await?;

        let routes = self.route_service.get_routes().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;
        let mut route_map = HashMap::with_capacity(routes.len());
        for r in routes {
            route_map.insert(r.get_id().unwrap(), r);
        }

        let connections = build_connections(&schedules, &route_map);

        let graph = self.route_service.get_route_map().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;
        let index_map = build_station_index(&graph);

        Ok((schedules, route_map, connections, graph, index_map))
    }
}

#[async_trait]
impl<RS, TR, TSR> TrainScheduleService for TrainScheduleServiceImpl<RS, TR, TSR>
where
    RS: RouteService + 'static + Send + Sync,
    TR: TrainRepository + 'static + Send + Sync,
    TSR: TrainScheduleRepository,
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

    async fn direct_schedules(
        &self,
        date: NaiveDate,
        pairs: &[(StationId, StationId)],
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        let schedules = self.get_schedules(date).await?;
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

        let want: HashSet<_> = pairs.iter().copied().collect();

        let mut result = Vec::new();

        let mut route_pos_cache: HashMap<RouteId, Arc<HashMap<StationId, usize>>> =
            HashMap::with_capacity(route_map_by_id.len());

        for schedule in &schedules {
            let route_id = schedule.route_id();

            let route = match route_map_by_id.get(&route_id) {
                Some(r) => r,
                None => continue,
            };

            let pos_map: Arc<HashMap<StationId, usize>> = match route_pos_cache.entry(route_id) {
                Entry::Occupied(o) => o.get().clone(),
                Entry::Vacant(v) => {
                    let mut m = HashMap::with_capacity(route.stops().len());
                    for (idx, stop) in route.stops().iter().enumerate() {
                        m.insert(stop.station_id(), idx);
                    }
                    v.insert(Arc::new(m)).clone()
                }
            };

            for &(from, to) in &want {
                if let (Some(&i_from), Some(&i_to)) = (pos_map.get(&from), pos_map.get(&to)) {
                    if i_from < i_to {
                        result.push(schedule.clone());
                        break;
                    }
                }
            }
        }

        result.sort_by_key(|s| s.origin_departure_time());
        Ok(result)
    }

    // 原本的想法是「交集‑筛站法」，但时间复杂度较高，达到 O(N · L) （N 为站点对数，L 为列车时刻表长度）
    // 故改用Connection Scan Algorithm（CSA）算法。详见https://arxiv.org/abs/1703.05997
    async fn transfer_schedules(
        &self,
        date: NaiveDate,
        pairs: &[(StationId, StationId)],
    ) -> Result<Vec<(Vec<TrainScheduleId>, Option<StationId>)>, TrainScheduleServiceError> {
        let (_, _, connections, graph, index_map) = self.load_daily_context(date).await?;

        let index_station_map: HashMap<usize, _> =
            index_map.iter().map(|(s, &i)| (i, *s)).collect();

        let mut transfer_solutions = Vec::new();

        for &(dep, arr) in pairs {
            let departure_index = index_map[&dep];
            let arrival_index = index_map[&arr];

            let (_, level1) = earliest_arrival_k1(
                departure_index,
                graph.node_count(),
                0,
                &connections,
                &index_map,
            );

            let (ids, mid_option) = backtrack_transfer(&level1, arrival_index, &index_station_map);

            if ids.len() == 2 && mid_option.is_some() {
                transfer_solutions.push((ids, mid_option));
            }
        }

        Ok(transfer_solutions)
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

    async fn get_terminal_arrival_time(
        &self,
        train_number: &str,
        origin_departure_time: &str,
    ) -> Result<Option<String>, TrainScheduleServiceError> {
        let origin_departure_time =
            DateTime::parse_from_rfc3339(origin_departure_time).map_err(|_| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    anyhow::anyhow!("Invalid origin departure time format"),
                ))
            })?;

        let date = origin_departure_time.date_naive();

        let schedules = self.get_schedules(date).await?;

        let trains = self.train_repository.get_trains().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;

        let train_id_to_number: HashMap<_, _> = trains
            .into_iter()
            .map(|t| (t.get_id().unwrap(), t.number().to_string()))
            .collect();

        let schedule = match schedules.iter().find(|s| {
            train_id_to_number
                .get(&s.train_id())
                .is_some_and(|num| num == train_number)
                && s.origin_departure_time()
                    == origin_departure_time.time().num_seconds_from_midnight() as i32
        }) {
            Some(s) => s,
            None => return Ok(None),
        };

        let routes = self.route_service.get_routes().await.map_err(|e| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                e.into(),
            ))
        })?;

        let route = routes
            .into_iter()
            .find(|r| r.get_id() == Some(schedule.route_id()))
            .ok_or_else(|| {
                TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                    anyhow::anyhow!("Route not found for train schedule"),
                ))
            })?;

        let terminal_stop = route.stops().last().ok_or_else(|| {
            TrainScheduleServiceError::InfrastructureError(ServiceError::RelatedServiceError(
                anyhow::anyhow!("No stops found in route"),
            ))
        })?;

        let terminal_station_id = terminal_stop.station_id();

        let arrival_time = self
            .get_station_arrival_time(schedule.get_id().unwrap(), terminal_station_id)
            .await?;

        Ok(Some(arrival_time.to_rfc3339()))
    }
}
