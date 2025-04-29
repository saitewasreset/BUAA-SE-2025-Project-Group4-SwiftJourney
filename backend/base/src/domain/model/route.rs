use crate::domain::model::station::StationId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;

define_id_type!(Stop);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stop {
    stop_id: Option<StopId>,
    station_id: StationId,
    arrival_time: u32,
    departure_time: u32,
    order: u32,
}

impl Identifiable for Stop {
    type ID = StopId;

    fn get_id(&self) -> Option<Self::ID> {
        self.stop_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.stop_id = Some(id)
    }
}

impl Entity for Stop {}

define_id_type!(Route);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    route_id: Option<RouteId>,
    stops: Vec<Stop>,
}

impl Identifiable for Route {
    type ID = RouteId;

    fn get_id(&self) -> Option<Self::ID> {
        self.route_id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.route_id = Some(id)
    }
}

impl Entity for Route {}

impl Aggregate for Route {}
