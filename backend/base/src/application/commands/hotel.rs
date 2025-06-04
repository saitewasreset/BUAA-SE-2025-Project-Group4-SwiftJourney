use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuotaQuery {
    pub session_id: String,
    pub hotel_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetType {
    City,
    Station,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotelQuery {
    pub session_id: String,
    pub target: String,
    pub target_type: TargetType,
    pub search: Option<String>,
    pub begin_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewCommentCommand {
    pub session_id: String,
    pub hotel_id: Uuid,
    pub rating: f64,
    pub comment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotelInfoQuery {
    pub session_id: String,
    pub hotel_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotelOrderInfoQuery {
    pub session_id: String,
    pub hotel_id: Uuid,
    pub begin_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}
