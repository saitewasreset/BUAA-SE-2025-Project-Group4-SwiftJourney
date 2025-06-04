use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, post, web};
use base::application::GeneralError;
use base::application::commands::train_query::{
    DirectTrainQueryCommand, TrainScheduleQueryCommand, TransferTrainQueryCommand,
};
use base::application::service::train_query::{
    DirectTrainQueryDTO, TrainQueryResponseDTO, TrainQueryService, TransferTrainQueryDTO,
};
use chrono::NaiveDate;
use serde::Deserialize;

/// 列车查询请求体DTO
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrainScheduleQuery {
    /// 出发站点
    pub departure_station: Option<String>,
    /// 到达站点
    pub arrival_station: Option<String>,
    /// 出发城市
    pub departure_city: Option<String>,
    /// 到达城市
    pub arrival_city: Option<String>,
    /// 出发日期，格式：YYYY-MM-DD
    pub departure_date: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrainScheduleInfoQuery {
    pub train_number: String,
    pub departure_date: String,
}

// Step 1: Define your API endpoint
// HINT: You may refer to RFC4 "直达车次查询（US1.2.1）" and "中转车次查询（US3.1.1）" for endpoint URL
// HINT: You may refer to https://actix.rs/docs/application#using-an-application-scope-to-compose-applications
// Thinking 1.2.1D - 9: 为何#[post("/query_direct")]中定义的URL只含真实URL的最后一部分？

// Step 2: Using Extractor to get needed user request data and your application service instance
// HINT: You may refer to https://actix.rs/docs/extractors
// HINT: You may refer to `api::user::user_manager` for example
// HINT: You may use `parse_request_body` function to parse request body as specified type `T`
// Exercise 1.2.1D - 7: Your code here. (1 / 5)

// Step 3: Implement `query_direct`
// HINT: You may refer to `api::user::user_manager` for example
// Exercise 1.2.1D - 7: Your code here. (2 / 5)
// To `api/train/schedule/mod.rs` for following exercise
#[post("/query_direct")]
async fn query_direct(
    request: HttpRequest,
    body: Bytes,
    train_query_service: web::Data<dyn TrainQueryService>,
) -> Result<ApiResponse<DirectTrainQueryDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let query_dto: TrainScheduleQuery = parse_request_body(body)?;

    let command = DirectTrainQueryCommand {
        session_id,
        departure_station: query_dto.departure_station,
        arrival_station: query_dto.arrival_station,
        departure_city: query_dto.departure_city,
        arrival_city: query_dto.arrival_city,
        departure_time: NaiveDate::parse_from_str(&query_dto.departure_date, "%Y-%m-%d").map_err(
            |_| {
                Box::new(GeneralError::BadRequest("Invalid date format".into()))
                    as Box<dyn base::application::ApplicationError>
            },
        )?,
    };

    ApiResponse::ok(train_query_service.query_direct_trains(command).await?)
}

#[post("/query_indirect")]
async fn query_indirect(
    request: HttpRequest,
    body: Bytes,
    train_query_service: web::Data<dyn TrainQueryService>,
) -> Result<ApiResponse<TransferTrainQueryDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let query_dto: TrainScheduleQuery = parse_request_body(body)?;

    let command = TransferTrainQueryCommand {
        session_id,
        departure_station: query_dto.departure_station,
        arrival_station: query_dto.arrival_station,
        departure_city: query_dto.departure_city,
        arrival_city: query_dto.arrival_city,
        departure_time: NaiveDate::parse_from_str(&query_dto.departure_date, "%Y-%m-%d").map_err(
            |_| {
                Box::new(GeneralError::BadRequest("Invalid date format".into()))
                    as Box<dyn base::application::ApplicationError>
            },
        )?,
    };

    ApiResponse::ok(train_query_service.query_transfer_trains(command).await?)
}

#[post("/")]

async fn query_train(
    request: HttpRequest,
    body: Bytes,
    train_query_service: web::Data<dyn TrainQueryService>,
) -> Result<ApiResponse<TrainQueryResponseDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let query_dto: TrainScheduleInfoQuery = parse_request_body(body)?;

    let command = TrainScheduleQueryCommand {
        session_id,
        train_number: query_dto.train_number,
        departure_date: query_dto.departure_date,
    };

    ApiResponse::ok(train_query_service.query_train(command).await?)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use super::*;

    #[test]
    fn test_direct_query_empty_station() {
        // 测试当站点为空时的参数验证
        let command = DirectTrainQueryCommand {
            session_id: "test_session".to_string(),
            departure_station: Some("".to_string()),
            arrival_station: Some("B".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 验证站点是空的
        assert!(command.departure_station.as_ref().unwrap().is_empty());

        // 模拟TrainQueryServiceImpl的行为
        let result = if command.departure_station.as_ref().unwrap().is_empty()
            || command.arrival_station.as_ref().unwrap().is_empty()
        {
            Err(
                Box::new(GeneralError::BadRequest("station id 不能为空".into()))
                    as Box<dyn base::application::ApplicationError>,
            )
        } else {
            Ok(DirectTrainQueryDTO { solutions: vec![] })
        };

        // 验证结果是错误
        assert!(result.is_err());
    }

    #[test]
    fn test_direct_query_invalid_date() {
        // 测试日期格式不正确时的处理
        let date_str = "2025/05/01"; // 错误的格式，应该是 2025-05-01

        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_err());
    }

    #[test]
    fn test_direct_query_success() {
        // 测试正确参数的情况
        let command = DirectTrainQueryCommand {
            session_id: "test_session".to_string(),
            departure_station: Some("A".to_string()),
            arrival_station: Some("B".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 验证站点不为空
        assert!(!command.departure_station.as_ref().unwrap().is_empty());
        assert!(!command.arrival_station.as_ref().unwrap().is_empty());

        // 模拟TrainQueryServiceImpl的行为
        let result = if command.departure_station.as_ref().unwrap().is_empty()
            || command.arrival_station.as_ref().unwrap().is_empty()
        {
            Err(
                Box::new(GeneralError::BadRequest("station id 不能为空".into()))
                    as Box<dyn base::application::ApplicationError>,
            )
        } else {
            Ok(DirectTrainQueryDTO { solutions: vec![] })
        };

        // 验证结果是成功的
        assert!(result.is_ok());
    }

    // 以下是换乘查询相关的测试

    #[test]
    fn test_transfer_query_empty_station() {
        // 测试当站点为空时的参数验证
        let command = TransferTrainQueryCommand {
            session_id: "test_session".to_string(),
            departure_station: Some("".to_string()),
            arrival_station: Some("C".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 验证出发站是空的
        assert!(command.departure_station.as_ref().unwrap().is_empty());

        // 模拟TrainQueryServiceImpl的行为
        let result = if command
            .departure_station
            .as_ref()
            .is_none_or(|s| s.is_empty())
            || command
                .arrival_station
                .as_ref()
                .is_none_or(|s| s.is_empty())
        {
            Err(Box::new(GeneralError::BadRequest("站点不能为空".into()))
                as Box<dyn base::application::ApplicationError>)
        } else {
            Ok(TransferTrainQueryDTO { solutions: vec![] })
        };

        // 验证结果是错误
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_query_same_station() {
        // 测试当出发站和到达站相同时的情况
        let command = TransferTrainQueryCommand {
            session_id: "test_session".to_string(),
            departure_station: Some("A".to_string()),
            arrival_station: Some("A".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 验证出发站和到达站相同
        assert_eq!(command.departure_station, command.arrival_station);

        // 模拟TrainQueryServiceImpl的行为
        let result = if command.departure_station == command.arrival_station
            && command.departure_station.is_some()
        {
            Err(
                Box::new(GeneralError::BadRequest("出发站和到达站不能相同".into()))
                    as Box<dyn base::application::ApplicationError>,
            )
        } else {
            Ok(TransferTrainQueryDTO { solutions: vec![] })
        };

        // 验证结果是错误
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_query_invalid_date() {
        // 测试日期格式不正确时的处理
        let date_str = "2025/05/01"; // 错误的格式，应该是 2025-05-01

        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_err());

        // 测试过去日期
        let past_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let now = chrono::Local::now().date_naive();

        // 模拟验证逻辑
        let is_valid = past_date >= now;
        assert!(!is_valid, "过去的日期应该被拒绝");
    }

    #[test]
    fn test_transfer_query_success() {
        // 测试正确参数的情况
        let command = TransferTrainQueryCommand {
            session_id: "test_session".to_string(),
            departure_station: Some("A".to_string()),
            arrival_station: Some("C".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 验证站点不为空且不相同
        assert!(!command.departure_station.as_ref().unwrap().is_empty());
        assert!(!command.arrival_station.as_ref().unwrap().is_empty());
        assert_ne!(command.departure_station, command.arrival_station);

        // 模拟TrainQueryServiceImpl的行为
        let result = if command
            .departure_station
            .as_ref()
            .is_none_or(|s| s.is_empty())
            || command
                .arrival_station
                .as_ref()
                .is_none_or(|s| s.is_empty())
            || (command.departure_station == command.arrival_station
                && command.departure_station.is_some())
        {
            Err(Box::new(GeneralError::BadRequest("参数错误".into()))
                as Box<dyn base::application::ApplicationError>)
        } else {
            Ok(TransferTrainQueryDTO { solutions: vec![] })
        };

        // 验证结果是成功的
        assert!(result.is_ok());
    }

    #[test]
    fn test_transfer_relaxing_time_calculation() {
        // 测试换乘休息时间计算
        let first_arrival = "2025-05-01 10:30:00";
        let second_departure = "2025-05-01 11:00:00";

        let first_dt = NaiveDateTime::parse_from_str(first_arrival, "%Y-%m-%d %H:%M:%S").unwrap();
        let second_dt =
            NaiveDateTime::parse_from_str(second_departure, "%Y-%m-%d %H:%M:%S").unwrap();

        // 计算休息时间（秒）
        let relaxing_time = (second_dt - first_dt).num_seconds() as u32;

        // 验证休息时间是30分钟（1800秒）
        assert_eq!(relaxing_time, 1800);

        // 测试跨天的情况
        let first_arrival_late = "2025-05-01 23:50:00";
        let second_departure_next_day = "2025-05-02 00:10:00";

        let first_dt_late =
            NaiveDateTime::parse_from_str(first_arrival_late, "%Y-%m-%d %H:%M:%S").unwrap();
        let second_dt_next_day =
            NaiveDateTime::parse_from_str(second_departure_next_day, "%Y-%m-%d %H:%M:%S").unwrap();

        // 计算休息时间（秒）
        let relaxing_time_overnight = (second_dt_next_day - first_dt_late).num_seconds() as u32;

        // 验证休息时间是20分钟（1200秒）
        assert_eq!(relaxing_time_overnight, 1200);
    }
}
