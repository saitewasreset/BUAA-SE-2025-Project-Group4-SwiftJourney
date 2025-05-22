#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CreateTrainOrderCommand{
    pub train_number: String,
    pub origin_departure_time: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub personal_id: String,
    pub seat_type: String,
}