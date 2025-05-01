use crate::domain::model::station::StationId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use id_macro::define_id_type;

define_id_type!(Stop);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stop {
    stop_id: Option<StopId>,
    route_id: Option<RouteId>,
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

impl Stop {
    pub fn new(
        id: Option<StopId>,
        route_id: Option<RouteId>,
        station_id: StationId,
        arrival_time: u32,
        departure_time: u32,
        order: u32,
    ) -> Self {
        Stop {
            stop_id: id,
            route_id,
            station_id,
            arrival_time,
            departure_time,
            order,
        }
    }

    pub fn station_id(&self) -> StationId {
        self.station_id
    }

    pub fn arrival_time(&self) -> u32 {
        self.arrival_time
    }

    pub fn departure_time(&self) -> u32 {
        self.departure_time
    }

    pub fn order(&self) -> u32 {
        self.order
    }
}

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
        self.route_id = Some(id);

        for stop in &mut self.stops {
            stop.route_id = Some(id);
        }
    }
}

impl Entity for Route {}

impl Aggregate for Route {}

impl Route {
    pub fn new(id: Option<RouteId>) -> Self {
        Route {
            route_id: id,
            stops: Vec::new(),
        }
    }

    pub fn stops(&self) -> &[Stop] {
        &self.stops
    }

    pub fn into_stops(self) -> Vec<Stop> {
        self.stops
    }

    pub fn add_stop(
        &mut self,
        stop_id: Option<StopId>,
        station_id: StationId,
        arrival_time: u32,
        departure_time: u32,
        order: u32,
    ) {
        let stop = Stop::new(
            stop_id,
            self.route_id,
            station_id,
            arrival_time,
            departure_time,
            order,
        );
        self.stops.push(stop);
    }
}
