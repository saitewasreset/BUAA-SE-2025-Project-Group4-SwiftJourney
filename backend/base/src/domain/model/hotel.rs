use crate::domain::Identifier;
use chrono::NaiveDate;
use id_macro::define_id_type;

define_id_type!(Hotel);
pub struct Hotel {}

pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

pub struct HotelRoom {}
