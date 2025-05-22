use crate::domain::model::route::Route;
use crate::domain::model::train_schedule::{Seat, SeatAvailabilityId, SeatLocationInfo};
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::seat_availability::{
    OccupiedSeatInfoMap, SeatAvailabilityRepository,
};
use crate::domain::service::ServiceError;
use crate::domain::service::train_seat::{TrainSeatService, TrainSeatServiceError};
use crate::domain::{DbId, Identifiable, RepositoryError};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TrainSeatServiceImpl<SAR, RR>
where
    SAR: SeatAvailabilityRepository,
    RR: RouteRepository,
{
    seat_availability_repository: Arc<SAR>,
    route_repository: Arc<RR>,
}

fn calc_station_id_to_order_map(route: &Route) -> HashMap<i32, u32> {
    route
        .stops()
        .iter()
        .map(|stop| (stop.station_id().to_db_value(), stop.order()))
        .collect()
}
fn calc_available_seat_count_map(
    seat_type_id: i32,
    occupied_seat_info_map: &OccupiedSeatInfoMap,
    station_id_to_order_map: &HashMap<i32, u32>,
    route_stops_count: usize,
) -> HashMap<(u32, u32), u32> {
    let mut result: HashMap<(u32, u32), u32> = HashMap::new();

    let mut seat_to_occupied_bitmap: HashMap<i64, Vec<bool>> = HashMap::new();

    let inner_map = occupied_seat_info_map
        .get(&seat_type_id)
        .expect("seat type id should in occupied_seat_info_map");

    for ((begin_station_id, end_station_id), seat_list) in inner_map {
        let begin_order = *station_id_to_order_map
            .get(begin_station_id)
            .expect("begin station id should in station id to order map");
        let end_order = *station_id_to_order_map
            .get(end_station_id)
            .expect("end station id should in station id to order map");

        for seat_id in seat_list {
            let occupied_bitmap = seat_to_occupied_bitmap
                .entry(*seat_id)
                .or_insert_with(|| vec![false; route_stops_count]);

            occupied_bitmap[begin_order as usize..end_order as usize].fill(true);
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Status {
        Begin,
        InOccupied,
        InAvailable,
    }

    for (_, occupied_bitmap) in seat_to_occupied_bitmap {
        let mut begin_order = 0;
        let mut end_order;

        let mut current_status = Status::Begin;

        for (i, occupied) in occupied_bitmap.iter().enumerate() {
            match current_status {
                Status::Begin => {
                    if *occupied {
                        current_status = Status::InOccupied;
                    } else {
                        current_status = Status::InAvailable;
                        begin_order = i;
                    }
                }
                Status::InAvailable => {
                    if *occupied {
                        current_status = Status::InOccupied;
                        end_order = i - 1;

                        let entry = result
                            .entry((begin_order as u32, end_order as u32))
                            .or_default();

                        *entry += 1;
                    }
                }
                Status::InOccupied => {
                    if !*occupied {
                        current_status = Status::InAvailable;
                        begin_order = i;
                    }
                }
            }
        }

        if current_status == Status::InAvailable {
            end_order = occupied_bitmap.len() - 1;

            let entry = result
                .entry((begin_order as u32, end_order as u32))
                .or_default();

            *entry += 1;
        }
    }

    result
}

impl<SAR, RR> TrainSeatServiceImpl<SAR, RR>
where
    SAR: SeatAvailabilityRepository,
    RR: RouteRepository,
{
}

#[async_trait]
impl<SAR, RR> TrainSeatService for TrainSeatServiceImpl<SAR, RR>
where
    SAR: SeatAvailabilityRepository,
    RR: RouteRepository,
{
    async fn available_seats_count(
        &self,
        seat_availability_id: SeatAvailabilityId,
    ) -> Result<u32, TrainSeatServiceError> {
        let seat_availability = self
            .seat_availability_repository
            .find(seat_availability_id)
            .await
            .context(format!(
                "failed to find seat availability for id: {}",
                seat_availability_id
            ))
            .map_err(|e| {
                TrainSeatServiceError::InfrastructureError(ServiceError::RepositoryError(e.into()))
            })?
            .ok_or(TrainSeatServiceError::InvalidSeatAvailability(
                seat_availability_id,
            ))?;

        let route = self
            .route_repository
            .get_by_train_schedule(seat_availability.train_schedule_id())
            .await
            .context(format!(
                "failed to get route for train schedule id: {}",
                seat_availability.train_schedule_id()
            ))
            .map_err(|e| {
                TrainSeatServiceError::InfrastructureError(ServiceError::RepositoryError(e.into()))
            })?
            .ok_or(TrainSeatServiceError::InfrastructureError(
                ServiceError::RepositoryError(RepositoryError::InconsistentState(anyhow!(
                    "no route for train schedule id: {}",
                    seat_availability.train_schedule_id()
                ))),
            ))?;

        let station_id_to_order_map = calc_station_id_to_order_map(&route);

        let occupied_seat_info_map = self
            .seat_availability_repository
            .get_train_schedule_occupied_seat(seat_availability.train_schedule_id())
            .await
            .context(format!(
                "failed to get occupied seat info map for train schedule id: {}",
                seat_availability.train_schedule_id()
            ))
            .map_err(|e| {
                TrainSeatServiceError::InfrastructureError(ServiceError::RepositoryError(e.into()))
            })?;

        let seat_type_id = seat_availability
            .seat_type()
            .get_id()
            .expect("seat type id should be present")
            .to_db_value();

        let route_stops_count = route.stops().len();

        let available_seat_count_map = calc_available_seat_count_map(
            seat_type_id,
            &occupied_seat_info_map,
            &station_id_to_order_map,
            route_stops_count,
        );

        let begin_order = *station_id_to_order_map
            .get(
                &seat_availability
                    .station_range()
                    .get_from_station_id()
                    .to_db_value(),
            )
            .expect("begin station should present in station_id_to_order_map");

        let end_order = *station_id_to_order_map
            .get(
                &seat_availability
                    .station_range()
                    .get_to_station_id()
                    .to_db_value(),
            )
            .expect("end station should present in station_id_to_order_map");

        Ok(available_seat_count_map
            .get(&(begin_order, end_order))
            .copied()
            .unwrap_or_default())
    }

    async fn reserve_seat(
        &self,
        seat_availability_id: SeatAvailabilityId,
        seat_location_info: SeatLocationInfo,
    ) -> Result<Seat, TrainSeatServiceError> {
        todo!()
    }

    async fn free_seat(
        &self,
        seat_availability_id: SeatAvailabilityId,
        seat: Seat,
    ) -> Result<(), TrainSeatServiceError> {
        todo!()
    }
}
