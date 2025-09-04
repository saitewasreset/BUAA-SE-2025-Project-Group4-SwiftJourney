use actix_web::{get, web};
use hotel_base::application::service::internal::HotelInternalService;
use shared::api::{ApiResponse, ApplicationError, ApplicationErrorBox};
use shared::internal::hotel::dto::{DbHotelDTO, DbHotelRoomTypeDTO};

#[get("/db_get_hotels")]
pub async fn db_get_hotels(
    hotel_internal_service: web::Data<dyn HotelInternalService>,
) -> Result<ApiResponse<Vec<DbHotelDTO>>, ApplicationErrorBox> {
    let result = hotel_internal_service
        .db_get_hotels()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_hotel_room_types")]
pub async fn db_get_hotel_room_types(
    hotel_internal_service: web::Data<dyn HotelInternalService>,
) -> Result<ApiResponse<Vec<DbHotelRoomTypeDTO>>, ApplicationErrorBox> {
    let result = hotel_internal_service
        .db_get_hotel_room_types()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(db_get_hotels).service(db_get_hotel_room_types);
}
