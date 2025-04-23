use super::{
    personal_info::PersonalInfoId,
    station::StationId,
    train::{SeatType, TrainId, Verified},
};
use crate::domain::model::train::Unverified;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use chrono::NaiveDate;
use id_macro::define_id_type;
use std::collections::HashMap;
use std::marker::PhantomData;

define_id_type!(TrainSchedule);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StationRange<State = Unverified>(StationId, StationId, PhantomData<State>);

pub type SeatAvailabilityMap =
    HashMap<StationRange<Verified>, HashMap<SeatType<Verified>, SeatAvailability>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainSchedule {
    id: Option<TrainScheduleId>,
    train_id: TrainId,
    date: NaiveDate,
    seat_availability: SeatAvailabilityMap,
}

impl Identifiable for TrainSchedule {
    type ID = TrainScheduleId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id)
    }
}

impl Entity for TrainSchedule {}
impl Aggregate for TrainSchedule {}

impl TrainSchedule {
    fn seat_availability(
        &self,
        seat_type: &SeatType<Verified>,
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
        seat_type: &SeatType<Verified>,
        station_range: &StationRange<Verified>,
    ) -> &mut SeatAvailability {
        self.seat_availability
            .get_mut(&station_range)
            .expect("seat_availability should contain verified station range")
            .get_mut(seat_type)
            .expect("seat_availability should contain verified seat type")
    }

    pub fn new(train_id: TrainId, date: NaiveDate, seat_availability: SeatAvailabilityMap) -> Self {
        Self {
            id: None,
            train_id,
            date,
            seat_availability,
        }
    }

    pub fn train_id(&self) -> TrainId {
        self.train_id
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn available_seats_count(
        &self,
        seat_type: &SeatType<Verified>,
        station_range: &StationRange<Verified>,
    ) -> Option<u32> {
        Some(
            self.seat_availability(seat_type, station_range)
                .available_seats_count(),
        )
    }

    pub fn seat_type(&self) -> Vec<SeatType<Verified>> {
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

        self.seat_availability_mut(seat_type, station_range)
            .add_occupied_seat(seat, passenger_id)
    }

    // 注意：若对应ID的座位未被占有，则不执行任何操作
    pub fn remove_occupied_seat(&mut self, station_range: &StationRange<Verified>, seat: Seat) {
        let seat_type = &seat.seat_type;

        self.seat_availability_mut(seat_type, station_range)
            .remove_occupied_seat(seat);
    }
}

define_id_type!(SeatAvailability);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatAvailability {
    id: Option<SeatAvailabilityId>,
    seat_type: SeatType<Verified>,
    from_station: StationId,
    to_station: StationId,
    occupied_seat: HashMap<SeatId, OccupiedSeat>,
}

impl SeatAvailability {
    pub fn available_seats_count(&self) -> u32 {
        self.seat_type.capacity() - self.occupied_seat.len() as u32
    }

    pub fn add_occupied_seat(&mut self, seat: Seat, passenger_id: PersonalInfoId) {
        self.occupied_seat.insert(
            seat.id,
            OccupiedSeat {
                id: None,
                seat,
                passenger_id,
            },
        );
    }

    pub fn remove_occupied_seat(&mut self, seat: Seat) {
        self.occupied_seat.remove(&seat.id);
    }

    pub fn get_available_seat() {
        todo!()
    }
}

impl Identifiable for SeatAvailability {
    type ID = SeatAvailabilityId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id)
    }
}

impl Entity for SeatAvailability {}

define_id_type!(OccupiedSeat);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccupiedSeat {
    id: Option<OccupiedSeatId>,
    seat: Seat,
    passenger_id: PersonalInfoId,
}

impl Identifiable for OccupiedSeat {
    type ID = OccupiedSeatId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for OccupiedSeat {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeatStatus {
    Available,
    Occupied,
}

define_id_type!(Seat);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Seat {
    id: SeatId,
    seat_type: SeatType<Verified>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeatLocationInfo {
    pub carriage: i32,
    pub row: i32,
    pub location: char,
}
