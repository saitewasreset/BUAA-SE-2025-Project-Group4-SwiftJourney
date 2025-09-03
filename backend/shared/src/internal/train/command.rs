use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GetTrainByNumberQuery {
    pub train_number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GetTrainScheduleQuery {
    pub train_id: u64,
    pub origin_departure_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GetTerminalArrivalTimeQuery {
    pub train_number: String,
    origin_departure_time: DateTimeWithTimeZone,
}
