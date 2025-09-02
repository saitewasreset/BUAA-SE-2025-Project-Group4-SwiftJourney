use actix_web::{
    post,
    web::{self, Bytes, Data},
    HttpRequest,
};
use chrono::NaiveDate;
use serde::Deserialize;
use shared::{
    api::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body},
    application_error::GeneralError,
};
use train_base::application::{
    commands::train_query::{
        DirectTrainQueryCommand, TrainScheduleQueryCommand, TransferTrainQueryCommand,
    },
    service::train_query::{
        DirectTrainQueryDTO, TrainQueryResponseDTO, TrainQueryService, TransferTrainQueryDTO,
    },
};

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

#[post("/query_direct")]
async fn query_direct(
    request: HttpRequest,
    body: Bytes,
    train_query_service: Data<dyn TrainQueryService>,
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
                    as Box<dyn shared::application_error::ApplicationError>
            },
        )?,
    };

    ApiResponse::ok(train_query_service.query_direct_trains(command).await?)
}

#[post("/query_indirect")]
async fn query_indirect(
    request: HttpRequest,
    body: Bytes,
    train_query_service: Data<dyn TrainQueryService>,
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
                    as Box<dyn shared::application_error::ApplicationError>
            },
        )?,
    };

    ApiResponse::ok(train_query_service.query_transfer_trains(command).await?)
}

#[post("/")]
async fn query_train(
    request: HttpRequest,
    body: Bytes,
    train_query_service: Data<dyn TrainQueryService>,
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

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_direct)
        .service(query_indirect)
        .service(query_train);
}
