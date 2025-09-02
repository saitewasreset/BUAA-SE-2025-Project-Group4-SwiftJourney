use shared::domain::model::train::{SeatType, Train};
use shared::domain::model::train_schedule::{StationRange, TrainSchedule};
use shared::domain::Identifiable;
use shared::internal::train::dto::{SeatTypeDTO, StationRangeDTO, TrainDTO, TrainScheduleDTO};

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
                .seat_availability_map
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
