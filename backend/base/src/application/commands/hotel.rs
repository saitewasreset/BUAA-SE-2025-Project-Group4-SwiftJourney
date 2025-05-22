use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuotaQuery {
    pub session_id: String,
    pub hotel_id: Uuid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewCommentCommand {
    pub session_id: String,
    pub hotel_id: Uuid,
    pub rating: f64,
    pub comment: String,
}
