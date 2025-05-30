use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DishQueryDTO {
    /// 车次
    pub train_number: String,
    /// 离开“始发站”的日期时间
    pub origin_departure_time: String,
}
