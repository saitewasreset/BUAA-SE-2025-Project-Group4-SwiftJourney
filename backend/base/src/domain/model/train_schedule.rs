use super::{
    personal_info::PersonalInfoId,
    station::StationId,
    train::{SeatType, TrainId},
};
use crate::domain::model::route::RouteId;
use crate::domain::model::train::SeatTypeId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use crate::{Unverified, Verified};
use chrono::NaiveDate;
use id_macro::define_id_type;
use std::collections::HashMap;
use std::marker::PhantomData;

define_id_type!(TrainSchedule);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StationRange<State = Unverified>(StationId, StationId, PhantomData<State>);

impl StationRange<Verified> {
    pub fn from_unchecked(
        from_station: StationId,
        to_station: StationId,
    ) -> StationRange<Verified> {
        StationRange(from_station, to_station, PhantomData)
    }
}

impl<T> StationRange<T> {
    pub fn get_from_station_id(&self) -> StationId {
        self.0
    }

    pub fn get_to_station_id(&self) -> StationId {
        self.1
    }
}

impl From<(StationId, StationId)> for StationRange<Unverified> {
    fn from(value: (StationId, StationId)) -> Self {
        Self(value.0, value.1, PhantomData)
    }
}

pub type SeatAvailabilityMap = HashMap<StationRange<Verified>, HashMap<SeatType, SeatAvailability>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainSchedule {
    id: Option<TrainScheduleId>,
    train_id: TrainId,
    date: NaiveDate,
    route_id: RouteId,
    seat_availability: SeatAvailabilityMap,
}

impl Identifiable for TrainSchedule {
    type ID = TrainScheduleId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
        self.update_occupied_seat_train_schedule_id();
    }
}

impl Entity for TrainSchedule {}
impl Aggregate for TrainSchedule {}

impl TrainSchedule {
    fn seat_availability(
        &self,
        seat_type: &SeatType,
        station_range: &StationRange<Verified>,
    ) -> &SeatAvailability {
        self.seat_availability
            .get(station_range)
            .expect("seat_availability should contain verified station range")
            .get(seat_type)
            .expect("seat_availability should contain verified seat type")
    }

    fn seat_availability_mut(
        &mut self,
        seat_type: &SeatType,
        station_range: &StationRange<Verified>,
    ) -> &mut SeatAvailability {
        self.seat_availability
            .get_mut(station_range)
            .expect("seat_availability should contain verified station range")
            .get_mut(seat_type)
            .expect("seat_availability should contain verified seat type")
    }

    fn update_occupied_seat_train_schedule_id(&mut self) {
        self.seat_availability
            .values_mut()
            .flat_map(|v| v.values_mut())
            .flat_map(|x| x.occupied_seat.values_mut())
            .for_each(|occupied_seat| {
                occupied_seat.train_schedule_id = self.id;

                if let Some(id) = occupied_seat.train_schedule_id {
                    occupied_seat.id = Some(OccupiedSeatId::new(
                        id,
                        occupied_seat.seat_type_id,
                        occupied_seat.seat.id,
                    ));
                }
            });
    }

    pub fn new(
        id: Option<TrainScheduleId>,
        train_id: TrainId,
        date: NaiveDate,
        route_id: RouteId,
        seat_availability: SeatAvailabilityMap,
    ) -> Self {
        Self {
            id,
            route_id,
            train_id,
            date,
            seat_availability,
        }
    }

    pub fn train_id(&self) -> TrainId {
        self.train_id
    }

    pub fn route_id(&self) -> RouteId {
        self.route_id
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn occupied_entry_iter(&self) -> impl Iterator<Item = &OccupiedSeat> {
        self.seat_availability
            .values()
            .flat_map(|x| x.values())
            .flat_map(|x| x.occupied_seat.values())
    }

    pub fn occupied_entry_len(&self) -> usize {
        self.occupied_entry_iter().count()
    }

    pub fn available_seats_count(
        &self,
        seat_type: &SeatType,
        station_range: &StationRange<Verified>,
    ) -> Option<u32> {
        Some(
            self.seat_availability(seat_type, station_range)
                .available_seats_count(),
        )
    }

    pub fn seat_type(&self) -> Vec<SeatType> {
        self.seat_availability
            .values()
            .next()
            .expect("SeatAvailabilityMap should have at least one element")
            .keys()
            .cloned()
            .collect()
    }

    pub fn get_seat_status_by_id(
        &self,
        station_range: &StationRange<Verified>,
        seat_id: SeatId,
    ) -> SeatStatus {
        match self
            .seat_availability
            .get(station_range)
            .expect("seat_availability should contain verified station range")
            .values()
            .flat_map(|x| x.occupied_seat.keys())
            .any(|e| *e == seat_id)
        {
            true => SeatStatus::Occupied,
            false => SeatStatus::Available,
        }
    }

    // 注意：若对应ID的座位已经被占有，则之前的占用将被移除
    pub fn add_occupied_seat(
        &mut self,
        station_range: &StationRange<Verified>,
        seat: Seat,
        passenger_id: PersonalInfoId,
    ) {
        let seat_type = &seat.seat_type;

        let train_schedule_id = self.id;

        self.seat_availability_mut(seat_type, station_range)
            .add_occupied_seat(train_schedule_id, seat, passenger_id)
    }

    // 注意：若对应ID的座位未被占有，则不执行任何操作
    pub fn remove_occupied_seat(&mut self, station_range: &StationRange<Verified>, seat: Seat) {
        let seat_type = &seat.seat_type;

        self.seat_availability_mut(seat_type, station_range)
            .remove_occupied_seat(seat);
    }

    pub fn into_seat_availability(self) -> Vec<SeatAvailability> {
        self.seat_availability
            .into_values()
            .flat_map(|x| x.into_values())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatAvailability {
    seat_type: SeatType,
    from_station: StationId,
    to_station: StationId,
    occupied_seat: HashMap<SeatId, OccupiedSeat>,
}

impl SeatAvailability {
    pub fn new(seat_type: SeatType, station_range: StationRange<Verified>) -> Self {
        Self {
            seat_type,
            from_station: station_range.get_from_station_id(),
            to_station: station_range.get_to_station_id(),
            occupied_seat: HashMap::new(),
        }
    }

    pub fn available_seats_count(&self) -> u32 {
        self.seat_type.capacity() - self.occupied_seat.len() as u32
    }

    pub fn add_occupied_seat(
        &mut self,
        train_schedule_id: Option<TrainScheduleId>,
        seat: Seat,
        passenger_id: PersonalInfoId,
    ) {
        self.occupied_seat.insert(
            seat.id,
            OccupiedSeat::new(
                train_schedule_id,
                self.seat_type.get_id().expect("seat_type_id should be set"),
                StationRange::from_unchecked(self.from_station, self.to_station),
                seat,
                passenger_id,
            ),
        );
    }

    pub fn remove_occupied_seat(&mut self, seat: Seat) {
        self.occupied_seat.remove(&seat.id);
    }

    pub fn seat_type(&self) -> &SeatType {
        &self.seat_type
    }

    pub fn station_range(&self) -> StationRange<Verified> {
        StationRange::from_unchecked(self.from_station, self.to_station)
    }

    pub fn into_occupied_seat(self) -> HashMap<SeatId, OccupiedSeat> {
        self.occupied_seat
    }

    pub fn get_available_seat() {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OccupiedSeatId {
    train_schedule_id: TrainScheduleId,
    seat_type_id: SeatTypeId,
    seat_id: SeatId,
}

impl OccupiedSeatId {
    pub fn new(
        train_schedule_id: TrainScheduleId,
        seat_type_id: SeatTypeId,
        seat_id: SeatId,
    ) -> Self {
        Self {
            train_schedule_id,
            seat_type_id,
            seat_id,
        }
    }

    pub fn train_schedule_id(&self) -> TrainScheduleId {
        self.train_schedule_id
    }

    pub fn seat_type_id(&self) -> SeatTypeId {
        self.seat_type_id
    }

    pub fn seat_id(&self) -> SeatId {
        self.seat_id
    }
}

impl Identifier for OccupiedSeatId {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccupiedSeat {
    id: Option<OccupiedSeatId>,
    train_schedule_id: Option<TrainScheduleId>,
    seat_type_id: SeatTypeId,
    station_range: StationRange<Verified>,
    seat: Seat,
    passenger_id: PersonalInfoId,
}

impl Identifiable for OccupiedSeat {
    type ID = OccupiedSeatId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id)
    }
}

impl Entity for OccupiedSeat {}

impl OccupiedSeat {
    pub fn new(
        train_schedule_id: Option<TrainScheduleId>,
        seat_type_id: SeatTypeId,
        station_range: StationRange<Verified>,
        seat: Seat,
        passenger_id: PersonalInfoId,
    ) -> Self {
        let id = train_schedule_id
            .map(|train_schedule_id| OccupiedSeatId::new(train_schedule_id, seat_type_id, seat.id));

        Self {
            id,
            train_schedule_id,
            seat_type_id,
            station_range,
            seat,
            passenger_id,
        }
    }

    pub fn seat(&self) -> &Seat {
        &self.seat
    }

    pub fn passenger_id(&self) -> PersonalInfoId {
        self.passenger_id
    }

    pub fn station_range(&self) -> StationRange<Verified> {
        self.station_range
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeatStatus {
    Available,
    Occupied,
}

define_id_type!(Seat);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Seat {
    id: SeatId,
    seat_type: SeatType,
    info: SeatLocationInfo,
    status: SeatStatus,
}

impl Identifiable for Seat {
    type ID = SeatId;

    fn get_id(&self) -> Option<Self::ID> {
        Some(self.id)
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}

impl Seat {
    pub fn new(id: SeatId, seat_type: SeatType, info: SeatLocationInfo) -> Self {
        Self {
            id,
            seat_type,
            info,
            status: SeatStatus::Available,
        }
    }

    pub fn seat_type(&self) -> &SeatType {
        &self.seat_type
    }

    pub fn location_info(&self) -> SeatLocationInfo {
        self.info
    }

    pub fn status(&self) -> SeatStatus {
        self.status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeatLocationInfo {
    pub carriage: i32,
    pub row: i32,
    pub location: char,
}
