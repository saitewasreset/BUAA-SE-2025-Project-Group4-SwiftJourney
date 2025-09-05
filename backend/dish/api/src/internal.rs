use actix_web::web::{Bytes, Data};
use actix_web::{get, post, web};
use dish_base::application::service::internal::DishInternalService;
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
use shared::application_error::GeneralError;
use shared::internal::dish::command::{SaveRawDishCommand, SaveRawTakeawayCommand};
use shared::internal::dish::dto::{DbDishDTO, DbTakeawayDishDTO, DbTakeawayShopDTO};
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

#[get("/db_get_dishes")]
pub async fn db_get_dishes(
    dish_internal_service: web::Data<dyn DishInternalService>,
) -> Result<ApiResponse<Vec<DbDishDTO>>, ApplicationErrorBox> {
    let result = dish_internal_service
        .db_get_dishes()
        .await
        .map_err(|_for_super_earth| {
            ApplicationErrorBox(GeneralError::InternalServerError.into())
        })?;

    ApiResponse::ok(result)
}

#[get("/db_get_takeaway_dishes")]
pub async fn db_get_takeaway_dishes(
    dish_internal_service: web::Data<dyn DishInternalService>,
) -> Result<ApiResponse<Vec<DbTakeawayDishDTO>>, ApplicationErrorBox> {
    let result = dish_internal_service
        .db_get_takeaway_dishes()
        .await
        .map_err(|_for_super_earth| {
            ApplicationErrorBox(GeneralError::InternalServerError.into())
        })?;

    ApiResponse::ok(result)
}

#[get("/db_get_takeaway_shops")]
pub async fn db_get_takeaway_shops(
    dish_internal_service: web::Data<dyn DishInternalService>,
) -> Result<ApiResponse<Vec<DbTakeawayShopDTO>>, ApplicationErrorBox> {
    let result = dish_internal_service
        .db_get_takeaway_shops()
        .await
        .map_err(|_for_super_earth| {
            ApplicationErrorBox(GeneralError::InternalServerError.into())
        })?;

    ApiResponse::ok(result)
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(save_raw_dish)
        .service(save_raw_takeaway)
        .service(db_get_dishes)
        .service(db_get_takeaway_dishes)
        .service(db_get_takeaway_shops);
}
