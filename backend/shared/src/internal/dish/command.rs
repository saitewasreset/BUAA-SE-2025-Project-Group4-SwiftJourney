use crate::data::{DishData, TakeawayData};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveRawDishCommand {
    pub dish: DishData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveRawTakeawayCommand {
    pub takeaway: TakeawayData,
}
