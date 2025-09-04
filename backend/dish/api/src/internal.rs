use actix_web::web::{Bytes, Data};
use actix_web::{post, web};
use dish_base::application::service::internal::DishInternalService;
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
use shared::application_error::GeneralError;
use shared::internal::dish::command::{SaveRawDishCommand, SaveRawTakeawayCommand};
use tracing::error;

#[post("/save_raw_dish")]
pub async fn save_raw_dish(
    body: Bytes,
    dish_internal_service: Data<dyn DishInternalService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let command: SaveRawDishCommand = parse_request_body(body)?;

    dish_internal_service
        .save_raw_dish(command)
        .await
        .inspect_err(|e| error!("Failed to save dish: {:?}", e))
        .map_err(|_for_super_earth| {
            ApplicationErrorBox(Box::from(GeneralError::InternalServerError))
        })?;

    ApiResponse::ok(())
}

#[post("/save_raw_takeaway")]
pub async fn save_raw_takeaway(
    body: Bytes,
    dish_internal_service: Data<dyn DishInternalService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let command: SaveRawTakeawayCommand = parse_request_body(body)?;

    dish_internal_service
        .save_raw_takeaway(command)
        .await
        .inspect_err(|e| error!("Failed to save takeaway: {:?}", e))
        .map_err(|_for_super_earth| {
            ApplicationErrorBox(Box::from(GeneralError::InternalServerError))
        })?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(save_raw_dish).service(save_raw_takeaway);
}
