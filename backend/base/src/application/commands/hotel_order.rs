use serde::{Deserialize, Serialize};

/// 酒店订单请求列表
pub type HotelOrderRequestsDTO = Vec<HotelOrderRequestDTO>;

/// 单个酒店预订请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotelOrderRequestDTO {
    /// 酒店UUID
    pub hotel_id: String,

    /// 房间类型（人类可读）
    pub room_type: String,

    /// 入住日期
    pub begin_date: Option<String>,

    /// 离开日期
    pub end_date: Option<String>,

    /// 预订人UUID
    pub personal_id: String,

    /// 预订数量
    pub amount: u32,
}
