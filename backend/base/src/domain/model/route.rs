use crate::domain::model::station::StationId;
use crate::domain::{Identifiable, Identifier};
use id_macro::define_id_type;

define_id_type!(Stop);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        self.stop_id = Some(id);
    }
}

impl Stop {
    pub fn station_id(&self) -> StationId {
        self.station_id
    }
}

define_id_type!(Route);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    route_id: RouteId,
    stops: Vec<Stop>,
}

impl Identifiable for Route {
    type ID = RouteId;

    fn get_id(&self) -> Option<Self::ID> {
        Some(self.route_id)
    }

    fn set_id(&mut self, id: Self::ID) {
        self.route_id = id;
    }
}

impl Route {
    pub fn stops(&self) -> impl Iterator<Item = &Stop> {
        self.stops.iter()
    }

    pub fn stop_pairs(&self) -> impl Iterator<Item = (&Stop, &Stop)> {
        self.stops.windows(2).map(|pair| (&pair[0], &pair[1]))
    }
}
