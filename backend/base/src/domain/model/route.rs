use crate::domain::Identifier;
use crate::domain::model::station::StationId;
use id_macro::define_id_type;

define_id_type!(Stop);
pub struct Stop {
    stop_id: Option<StopId>,
    station_id: StationId,
    arrival_time: u32,
    departure_time: u32,
    order: u32,
}

define_id_type!(Route);
pub struct Route {
    route_id: RouteId,
    stops: Vec<Stop>,
}
