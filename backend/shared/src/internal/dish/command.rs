use crate::internal::dish::dto::TrainDishOrderRequestDTO;

pub struct SaveRawDishCommand {}

pub struct SaveRawTakeawayCommand {}

pub struct OrderTrainDishCommand {
    pub session_id: String,
    pub info: TrainDishOrderRequestDTO,
}
