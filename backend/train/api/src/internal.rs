use actix_web::{
    get, post,
    web::{self, Bytes, Data},
};
use chrono::{DateTime, FixedOffset};
use shared::internal::train::command::{GetTerminalArrivalTimeQuery, VerifyTrainNumberQuery};
use shared::internal::train::dto::{
    DbRouteDTO, DbSeatTypeDTO, DbSeatTypeMappingDTO, DbTrainDTO, DbTrainScheduleDTO,
};
use shared::{
    api::{ApiResponse, ApplicationErrorBox, parse_request_body},
    application_error::ApplicationError,
    internal::train::{
        command::{GetTrainByNumberQuery, GetTrainScheduleQuery},
        dto::{TrainDTO, TrainScheduleDTO},
    },
};
use train_base::application::service::internal::TrainInternalService;

#[post("/get_train_by_number")]
pub async fn get_train_by_number(
    body: Bytes,
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Option<TrainDTO>>, ApplicationErrorBox> {
    let query: GetTrainByNumberQuery = parse_request_body(body)?;

    let result = train_internal_service
        .get_train_by_number(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/get_train_schedule")]
pub async fn get_train_schedule(
    body: Bytes,
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Option<TrainScheduleDTO>>, ApplicationErrorBox> {
    let query: GetTrainScheduleQuery = parse_request_body(body)?;

    let result = train_internal_service
        .get_train_schedule(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/get_terminal_arrival_time")]
pub async fn get_terminal_arrival_time(
    body: Bytes,
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<DateTime<FixedOffset>>, ApplicationErrorBox> {
    let query: GetTerminalArrivalTimeQuery = parse_request_body(body)?;

    let result = train_internal_service
        .get_terminal_arrival_time(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/get_trains")]
pub async fn get_trains(
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Vec<TrainDTO>>, ApplicationErrorBox> {
    let result = train_internal_service
        .get_trains()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/verify_train_number")]
pub async fn verify_train_number(
    body: Bytes,
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<bool>, ApplicationErrorBox> {
    let query: VerifyTrainNumberQuery = parse_request_body(body)?;

    let result = train_internal_service
        .verify_train_number(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_trains")]
pub async fn db_get_trains(
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Vec<DbTrainDTO>>, ApplicationErrorBox> {
    let result = train_internal_service
        .db_get_trains()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_routes")]
pub async fn db_get_routes(
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Vec<DbRouteDTO>>, ApplicationErrorBox> {
    let result = train_internal_service
        .db_get_routes()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_train_schedules")]
pub async fn db_get_train_schedules(
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Vec<DbTrainScheduleDTO>>, ApplicationErrorBox> {
    let result = train_internal_service
        .db_get_train_schedule()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_seat_types")]
pub async fn db_get_seat_types(
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Vec<DbSeatTypeDTO>>, ApplicationErrorBox> {
    let result = train_internal_service
        .db_get_seat_type()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_seat_type_mappings")]
pub async fn db_get_seat_type_mappings(
    train_internal_service: Data<dyn TrainInternalService>,
) -> Result<ApiResponse<Vec<DbSeatTypeMappingDTO>>, ApplicationErrorBox> {
    let result = train_internal_service
        .db_get_seat_type_mapping()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_train_by_number)
        .service(get_train_schedule)
        .service(get_trains)
        .service(verify_train_number)
        .service(db_get_trains)
        .service(db_get_routes)
        .service(db_get_train_schedules)
        .service(db_get_seat_types)
        .service(db_get_seat_type_mappings);
}
