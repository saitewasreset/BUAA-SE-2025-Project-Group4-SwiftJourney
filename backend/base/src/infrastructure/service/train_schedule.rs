use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::domain::Identifiable;
use crate::domain::model::route::{Route, RouteId};
use crate::domain::model::station::StationId;
use crate::domain::model::train::TrainId;
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::service::ServiceError;
use crate::domain::service::route::{RouteGraph, RouteService};
use crate::domain::service::train_schedule::{TrainScheduleService, TrainScheduleServiceError};
use async_trait::async_trait;
use chrono::{FixedOffset, NaiveDate};

// Step 1: Define generics parameter over `RouteService` service
// Exercise 1.2.1D - 3: Your code here. (1 / 6)
pub struct TrainScheduleServiceImpl<T>
where
    T: RouteService + 'static + Send + Sync,
{
    // Step 2: Add struct filed to store an implementation of `RouteService` service
    // Exercise 1.2.1D - 3: Your code here. (2 / 6)
    route_service: Arc<T>,
    tz_offset_hour: i32,
}

impl<T> TrainScheduleServiceImpl<T>
where
    T: RouteService + 'static + Send + Sync,
{
    pub fn new(route_service: Arc<T>, tz_offset_hour: i32) -> Self {
        Self {
            route_service,
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

impl<T> TrainScheduleServiceImpl<T>
where
    T: RouteService + 'static + Send + Sync,
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
impl<T> TrainScheduleService for TrainScheduleServiceImpl<T>
where
    T: RouteService + 'static + Send + Sync,
{
    async fn add_schedule(
        &self,
        train_id: TrainId,
        date: NaiveDate,
    ) -> Result<(), TrainScheduleServiceError> {
        todo!()
    }

    async fn get_schedules(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        todo!()
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
        let (schedules, _, connections, graph, index_map) = self.load_daily_context(date).await?;

        let lookup: HashMap<_, _> = schedules
            .iter()
            .filter_map(|s| s.get_id().map(|id| (id, s.clone())))
            .collect();

        let mut res = Vec::<TrainSchedule>::new();
        let mut seen = HashSet::<TrainScheduleId>::new();

        for &(dep, arr) in pairs {
            let departure_index = index_map[&dep];
            let arrival_index = index_map[&arr];

            let (level0, _) = earliest_arrival_k1(
                departure_index,
                graph.node_count(),
                0, // 起点时间：当天 00:00
                &connections,
                &index_map,
            );

            for schedule_id in backtrack_direct(&level0, arrival_index) {
                if seen.insert(schedule_id) {
                    if let Some(s) = lookup.get(&schedule_id) {
                        res.push(s.clone());
                    }
                }
            }
        }
        Ok(res)
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
}
