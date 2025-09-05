use crate::domain::Identifiable;
use crate::domain::model::train::{SeatType, Train};
use crate::domain::model::train_schedule::{StationRange, TrainSchedule};
use chrono::NaiveDate;
use sea_orm::prelude::{Date, Decimal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainDTO {
    pub id: u64,
    pub number: String,
    pub train_type: String,
    pub seats: HashMap<String, SeatTypeDTO>,
    pub default_route_id: u64,
    pub default_origin_departure_time: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SeatTypeDTO {
    pub seat_type_id: u64,
    pub type_name: String,
    pub capacity: u32,
    pub price: Decimal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainScheduleDTO {
    pub id: u64,
    pub train_id: u64,
    pub date: NaiveDate,
    pub origin_departure_time: i32,
    pub route_id: u64,
    pub seat_availability_map: Vec<(StationRangeDTO, Vec<(SeatTypeDTO, u64)>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StationRangeDTO {
    pub from_station_id: u64,
    pub to_station_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbTrainDTO {
    pub id: i32,
    pub number: String,
    pub type_id: i32,
    pub default_origin_departure_time: i32,
    pub default_line_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbRouteDTO {
    pub id: i32,
    pub line_id: i64,
    pub station_id: i32,
    pub arrival_time: i32,
    pub departure_time: i32,
    pub order: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbTrainScheduleDTO {
    pub id: i32,
    pub train_id: i32,
    pub departure_date: Date,
    pub origin_departure_time: i32,
    pub line_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbSeatTypeDTO {
    pub id: i32,
    pub type_name: String,
    pub capacity: i32,
    pub price: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbSeatTypeMappingDTO {
    pub train_type_id: i32,
    pub seat_type_id: i32,
    pub seat_id: i64,
    pub carriage: i32,
    pub row: i32,
    pub location: String,
}

impl From<Train> for TrainDTO {
    fn from(value: Train) -> Self {
        TrainDTO {
            id: value.get_id().unwrap().into(),
            number: value.number().to_string(),
            train_type: value.train_type().to_string(),
            seats: value
                .seats()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone().into()))
                .collect(),
            default_route_id: value.default_route_id().into(),
            default_origin_departure_time: value.default_origin_departure_time(),
        }
    }
}

impl From<SeatType> for SeatTypeDTO {
    fn from(value: SeatType) -> Self {
        SeatTypeDTO {
            seat_type_id: value.get_id().unwrap().into(),
            type_name: value.name().to_string(),
            capacity: value.capacity(),
            price: value.unit_price(),
        }
    }
}

impl From<TrainSchedule> for TrainScheduleDTO {
    fn from(value: TrainSchedule) -> Self {
        TrainScheduleDTO {
            id: value.get_id().unwrap().into(),
            train_id: value.train_id().into(),
            date: value.date(),
            origin_departure_time: value.origin_departure_time(),
            route_id: value.route_id().into(),
            seat_availability_map: value
                .seat_availability_map()
                .clone()
                .into_iter()
                .map(|(k, v)| {
                    let new_v = v
                        .into_iter()
                        .map(|(seat_type, seat_availability_id)| {
                            (seat_type.into(), seat_availability_id.into())
                        })
                        .collect();
                    (k.into(), new_v)
                })
                .collect(),
        }
    }
}

impl<T> From<StationRange<T>> for StationRangeDTO {
    fn from(value: StationRange<T>) -> Self {
        StationRangeDTO {
            from_station_id: value.get_from_station_id().into(),
            to_station_id: value.get_to_station_id().into(),
        }
    }
}
