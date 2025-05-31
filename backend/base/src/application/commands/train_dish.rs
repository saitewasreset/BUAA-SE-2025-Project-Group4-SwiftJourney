use crate::application::service::train_dish::TrainDishOrderRequestDTO;

pub struct OrderTrainDishCommand {
    pub session_id: String,
    pub info: TrainDishOrderRequestDTO,
}
