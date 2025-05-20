use crate::{ApiResponse, ApplicationErrorBox, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{post, web};
use base::application::GeneralError;
use base::application::commands::train_query::DirectTrainQueryCommand;
use base::application::service::train_query::{DirectTrainQueryDTO, TrainQueryService};
use chrono::NaiveDate;
use serde::Deserialize;

/// 列车查询请求体DTO
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrainScheduleQuery {
    /// 会话ID
    pub session_id: String,
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
    body: Bytes,
    train_query_service: web::Data<dyn TrainQueryService>,
) -> Result<ApiResponse<DirectTrainQueryDTO>, ApplicationErrorBox> {
    let query_dto: TrainScheduleQuery = parse_request_body(body)?;

    let command = DirectTrainQueryCommand {
        session_id: query_dto.session_id,
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

// #[post("/query_indirect")]
// async fn query_indirect(
//     body: Bytes,
//     train_query_service: web::Data<dyn TrainQueryService>,
// ) -> Result<ApiResponse<TransferTrainQueryDTO>, ApplicationErrorBox> {
//     let query_dto: TrainScheduleQuery = parse_request_body(body)?;

//     let command = TransferTrainQueryCommand {
//         session_id: query_dto.session_id,
//         from_city_id: query_dto.departureStation.unwrap_or_default(),
//         to_city_id: query_dto.arrivalStation.unwrap_or_default(),
//         departure_time: NaiveDate::parse_from_str(&query_dto.departureDate, "%Y-%m-%d").map_err(
//             |_| {
//                 Box::new(GeneralError::BadRequest("Invalid date format".into()))
//                     as Box<dyn base::application::ApplicationError>
//             },
//         )?,
//     };

//     ApiResponse::ok(train_query_service.query_transfer_trains(command).await?)
// }

#[cfg(test)]
mod tests {
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
}
