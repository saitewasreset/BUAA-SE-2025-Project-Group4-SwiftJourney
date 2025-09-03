use actix_web::web::Bytes;
use actix_web::{post, web};
use order_base::application::service::internal::OrderInternalService;
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
use shared::application_error::ApplicationError;
use shared::internal::order::command::{
    NewTransactionCommand, RefundTransactionCommand, UpdateOrdersCommand, UserOrderListQuery,
    VerifyTrainOrderQuery,
};
use shared::internal::order::dto::InternalOrderDTO;
use uuid::Uuid;

#[post("/new_transaction")]
pub async fn new_transaction(
    body: Bytes,
    order_internal_service: web::Data<dyn OrderInternalService>,
) -> Result<ApiResponse<Uuid>, ApplicationErrorBox> {
    let command: NewTransactionCommand = parse_request_body(body)?;

    let result = order_internal_service
        .new_transaction(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/refund_transaction")]
pub async fn refund_transaction(
    body: Bytes,
    order_internal_service: web::Data<dyn OrderInternalService>,
) -> Result<ApiResponse<Uuid>, ApplicationErrorBox> {
    let command: RefundTransactionCommand = parse_request_body(body)?;

    let result = order_internal_service
        .refund_transaction(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/verify_train_order")]
pub async fn verify_train_order(
    body: Bytes,
    order_internal_service: web::Data<dyn OrderInternalService>,
) -> Result<ApiResponse<bool>, ApplicationErrorBox> {
    let query: VerifyTrainOrderQuery = parse_request_body(body)?;

    let result = order_internal_service
        .verify_train_order(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/update_orders")]
pub async fn update_orders(
    body: Bytes,
    order_internal_service: web::Data<dyn OrderInternalService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let command: UpdateOrdersCommand = parse_request_body(body)?;

    order_internal_service
        .update_orders(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(())
}

#[post("/get_order_list_by_user_id")]
pub async fn get_order_list_by_user_id(
    body: Bytes,
    order_internal_service: web::Data<dyn OrderInternalService>,
) -> Result<ApiResponse<Vec<InternalOrderDTO>>, ApplicationErrorBox> {
    let query: UserOrderListQuery = parse_request_body(body)?;

    let result = order_internal_service
        .get_order_list_by_user_id(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

/// Registers all the API services in this module.
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(new_transaction)
        .service(refund_transaction)
        .service(verify_train_order)
        .service(update_orders)
        .service(get_order_list_by_user_id);
}
