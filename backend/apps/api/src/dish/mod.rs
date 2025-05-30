use actix_web::{
    HttpRequest, post,
    web::{self, Bytes},
};
use base::application::{commands::dish_query::DishQueryDTO, service::dish_query::{DishQueryService, TrainDishInfoDTO}};

use crate::{get_session_id, parse_request_body, ApiResponse, ApplicationErrorBox};

#[post("/dish/query")]
pub async fn query_dish(
    request: HttpRequest,
    body: Bytes,
    dish_query_service: web::Data<dyn DishQueryService>,
) -> Result<ApiResponse<TrainDishInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let query: DishQueryDTO = parse_request_body(body)?;

    ApiResponse::ok(dish_query_service.query_dish(query, session_id).await?)
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_dish);
}
