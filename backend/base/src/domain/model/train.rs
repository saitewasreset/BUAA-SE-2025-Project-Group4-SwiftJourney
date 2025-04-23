use crate::domain::model::route::RouteId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;
use sea_orm::prelude::Decimal;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;

define_id_type!(Train);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Train {
    id: Option<TrainId>,
    number: TrainNumber<Verified>,
    train_type: TrainType<Verified>,
    seats: HashMap<String, SeatType<Verified>>,
    route_id: RouteId,
}

impl Identifiable for Train {
    type ID = TrainId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for Train {}
impl Aggregate for Train {}

impl Train {
    pub fn new(
        number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seats: HashMap<String, SeatType<Verified>>,
        route_id: RouteId,
    ) -> Self {
        Train {
            id: None,
            number,
            train_type,
            seats,
            route_id,
        }
    }

    pub fn number(&self) -> &str {
        &self.number
    }

    pub fn train_type(&self) -> &str {
        &self.train_type
    }
}

define_id_type!(SeatType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Unverified;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Verified;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeatType<State = Unverified> {
    seat_type_id: Option<SeatTypeId>,
    type_name: String,
    capacity: u32,
    price: Decimal,
    _for_super_earth: PhantomData<State>,
}

impl SeatType<Verified> {
    pub fn name(&self) -> &str {
        &self.type_name
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn unit_price(&self) -> Decimal {
        self.price
    }
}

impl Identifiable for SeatType {
    type ID = SeatTypeId;

    fn get_id(&self) -> Option<Self::ID> {
        self.seat_type_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.seat_type_id = Some(id)
    }
}

impl Entity for SeatType {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrainType<State = Unverified>(String, PhantomData<State>);

impl From<String> for TrainType<Unverified> {
    fn from(value: String) -> Self {
        TrainType(value, PhantomData)
    }
}

impl<T> Deref for TrainType<T> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrainNumber<State = Unverified>(String, PhantomData<State>);

impl From<String> for TrainNumber<Unverified> {
    fn from(value: String) -> Self {
        TrainNumber(value, PhantomData)
    }
}

impl<T> Deref for TrainNumber<T> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
