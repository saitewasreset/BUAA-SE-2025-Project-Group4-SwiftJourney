use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::application::service::train_query::TrainQueryServiceError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainScheduleQueryCommand {
    pub session_id: String,
    pub train_number: String,
    pub departure_date: String,
}

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

pub trait TrainQueryValidate {
    fn dep_station(&self) -> &Option<String>;
    fn dep_city(&self) -> &Option<String>;
    fn arr_station(&self) -> &Option<String>;
    fn arr_city(&self) -> &Option<String>;

    fn validate(&self) -> Result<(), TrainQueryServiceError> {
        // —— 始发端 ——
        let dep_station = self
            .dep_station()
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let dep_city = self
            .dep_city()
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if dep_station == dep_city {
            return Err(TrainQueryServiceError::InconsistentQuery);
        }

        // —— 到达端 ——
        let arr_station = self
            .arr_station()
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let arr_city = self
            .arr_city()
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if arr_station == arr_city {
            return Err(TrainQueryServiceError::InconsistentQuery);
        }

        Ok(())
    }
}

impl TrainQueryValidate for DirectTrainQueryCommand {
    fn dep_station(&self) -> &Option<String> {
        &self.departure_station
    }
    fn dep_city(&self) -> &Option<String> {
        &self.departure_city
    }
    fn arr_station(&self) -> &Option<String> {
        &self.arrival_station
    }
    fn arr_city(&self) -> &Option<String> {
        &self.arrival_city
    }
}

impl TrainQueryValidate for TransferTrainQueryCommand {
    fn dep_station(&self) -> &Option<String> {
        &self.departure_station
    }
    fn dep_city(&self) -> &Option<String> {
        &self.departure_city
    }
    fn arr_station(&self) -> &Option<String> {
        &self.arrival_station
    }
    fn arr_city(&self) -> &Option<String> {
        &self.arrival_city
    }
}
