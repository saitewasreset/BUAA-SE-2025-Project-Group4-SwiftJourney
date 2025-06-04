use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::{
    HttpRequest, post,
    web::{self, Bytes},
};
use base::application::commands::train_dish::OrderTrainDishCommand;
use base::application::service::train_dish::{
    TrainDishApplicationService, TrainDishOrderRequestDTO,
};
use base::application::service::transaction::TransactionInfoDTO;
use base::application::{
    commands::dish_query::DishQueryDTO,
    service::dish_query::{DishQueryService, TrainDishInfoDTO},
};

#[post("/query")]
pub async fn query_dish(
    request: HttpRequest,
    body: Bytes,
    dish_query_service: web::Data<dyn DishQueryService>,
) -> Result<ApiResponse<TrainDishInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let query: DishQueryDTO = parse_request_body(body)?;

    ApiResponse::ok(dish_query_service.query_dish(query, session_id).await?)
}

#[post("/order")]
pub async fn order_dish(
    request: HttpRequest,
    body: Bytes,
    train_dish_application_service: web::Data<dyn TrainDishApplicationService>,
) -> Result<ApiResponse<TransactionInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let order_request: TrainDishOrderRequestDTO = parse_request_body(body)?;

    let command = OrderTrainDishCommand {
        session_id,
        info: order_request,
    };

    ApiResponse::ok(train_dish_application_service.order_dish(command).await?)
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_dish).service(order_dish);
}
