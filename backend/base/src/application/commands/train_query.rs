use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 直达车次查询（US1.2.1）——Query
/// 跨层传输时使用 `Serialize / Deserialize` 方便直接反序列化 JSON。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectTrainQueryCommand {
    /// 客户端会话，用于校验登录状态
    pub session_id: String,
    /// 始发站
    pub departure_station: Option<String>,
    /// 终到站
    pub arrival_station: Option<String>,
    /// 始发城市
    pub departure_city: Option<String>,
    /// 终到城市
    pub arrival_city: Option<String>,
    /// 乘车时间
    pub departure_time: NaiveDate,
}

/// 中转车次查询（US3.1.1）——Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTrainQueryCommand {
    pub session_id: String,
    /// 始发城市
    pub from_city: String,
    /// 终到城市
    pub to_city: String,
    pub departure_time: NaiveDate,
}
