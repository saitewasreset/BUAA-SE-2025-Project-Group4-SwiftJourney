use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::application::{ApplicationError, commands::dish_query::DishQueryDTO};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DishInfoDTO {
    pub available_time: Vec<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub dish_type: String,
    pub picture: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TakeawayDishInfoDTO {
    pub name: String,
    pub picture: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TakeawayDTO {
    pub shop_name: String,
    pub dishes: Vec<TakeawayDishInfoDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrainDishInfoDTO {
    pub train_number: String,
    pub origin_departure_time: String,
    pub terminal_arrival_time: String,

    pub dishes: Vec<DishInfoDTO>,

    pub takeaway: HashMap<String, Vec<TakeawayDTO>>,

    pub can_booking: bool,
    pub reason: Option<String>,
}

#[async_trait]
pub trait DishQueryService {
    async fn query_dish(
        &self,
        query: DishQueryDTO,
        session_id: String,
    ) -> Result<TrainDishInfoDTO, Box<dyn ApplicationError>>;
}
