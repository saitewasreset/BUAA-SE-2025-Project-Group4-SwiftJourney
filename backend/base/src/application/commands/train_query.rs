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


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn sample_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
    }

    // ------------------------------
    // DirectTrainQueryCommand 测试
    // ------------------------------

    #[test]
    fn test_direct_validate_positive() {
        let cmd = DirectTrainQueryCommand {
            session_id: "s1".to_string(),
            departure_station: Some("北京南".to_string()),
            arrival_station: Some("上海虹桥".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: sample_date(),
        };

        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_direct_validate_negative_inconsistent_departure() {
        let cmd = DirectTrainQueryCommand {
            session_id: "s2".to_string(),
            departure_station: Some("北京南".to_string()),
            arrival_station: Some("上海虹桥".to_string()),
            departure_city: Some("北京".to_string()),
            arrival_city: None,
            departure_time: sample_date(),
        };

        let result = cmd.validate();
        assert!(matches!(result, Err(TrainQueryServiceError::InconsistentQuery)));
    }

    #[test]
    fn test_direct_getters() {
        let cmd = DirectTrainQueryCommand {
            session_id: "s3".to_string(),
            departure_station: Some("A".to_string()),
            arrival_station: Some("B".to_string()),
            departure_city: Some("C".to_string()),
            arrival_city: Some("D".to_string()),
            departure_time: sample_date(),
        };

        assert_eq!(cmd.dep_station().as_deref(), Some("A"));
        assert_eq!(cmd.dep_city().as_deref(), Some("C"));
        assert_eq!(cmd.arr_station().as_deref(), Some("B"));
        assert_eq!(cmd.arr_city().as_deref(), Some("D"));
    }

    // ------------------------------
    // TransferTrainQueryCommand 测试
    // ------------------------------

    #[test]
    fn test_transfer_validate_positive() {
        let cmd = TransferTrainQueryCommand {
            session_id: "s4".to_string(),
            departure_station: None,
            arrival_station: Some("南京南".to_string()),
            departure_city: Some("北京".to_string()),
            arrival_city: None,
            departure_time: sample_date(),
        };

        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_transfer_validate_negative_inconsistent_arrival() {
        let cmd = TransferTrainQueryCommand {
            session_id: "s5".to_string(),
            departure_station: None,
            arrival_station: Some("广州南".to_string()),
            departure_city: Some("北京".to_string()),
            arrival_city: Some("广州".to_string()),
            departure_time: sample_date(),
        };

        let result = cmd.validate();
        assert!(matches!(result, Err(TrainQueryServiceError::InconsistentQuery)));
    }

    #[test]
    fn test_transfer_getters() {
        let cmd = TransferTrainQueryCommand {
            session_id: "s6".to_string(),
            departure_station: Some("X".to_string()),
            arrival_station: Some("Y".to_string()),
            departure_city: Some("M".to_string()),
            arrival_city: Some("N".to_string()),
            departure_time: sample_date(),
        };

        assert_eq!(cmd.dep_station().as_deref(), Some("X"));
        assert_eq!(cmd.dep_city().as_deref(), Some("M"));
        assert_eq!(cmd.arr_station().as_deref(), Some("Y"));
        assert_eq!(cmd.arr_city().as_deref(), Some("N"));
    }
}
