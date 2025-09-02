use actix_web::{
    post,
    web::{self, Bytes, Data},
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

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_train_by_number)
        .service(get_train_schedule);
}
