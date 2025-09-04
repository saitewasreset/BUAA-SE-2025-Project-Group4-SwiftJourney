use actix_web::web::Bytes;
use actix_web::{get, post, web};
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
use shared::application_error::ApplicationError;
use shared::internal::user::command::{
    ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
    VerifyPasswordCommand, VerifyPaymentPasswordCommand,
};
use shared::internal::user::dto::{DbPersonalInfo, DbUserDTO, SessionDTO, UserCombinedInfoDTO};
use user_base::application::service::internal::UserInternalService;

#[post("/verify_password")]
pub async fn verify_password(
    body: Bytes,
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<bool>, ApplicationErrorBox> {
    let command: VerifyPasswordCommand = parse_request_body(body)?;

    let result = user_internal_service
        .verify_password(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/verify_payment_password")]
pub async fn verify_payment_password(
    body: Bytes,
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<bool>, ApplicationErrorBox> {
    let command: VerifyPaymentPasswordCommand = parse_request_body(body)?;

    let result = user_internal_service
        .verify_payment_password(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/set_payment_password")]
pub async fn set_payment_password(
    body: Bytes,
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let command: SetPaymentPasswordCommand = parse_request_body(body)?;

    user_internal_service
        .set_payment_password(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(())
}

#[post("/clear_wrong_payment_password_tried")]
pub async fn clear_wrong_payment_password_tried(
    body: Bytes,
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let command: ClearWrongPaymentPasswordTriedCommand = parse_request_body(body)?;

    user_internal_service
        .clear_wrong_payment_password_tried(command)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(())
}

#[post("/get_session")]
pub async fn get_session(
    body: Bytes,
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<Option<SessionDTO>>, ApplicationErrorBox> {
    let query: SessionQuery = parse_request_body(body)?;

    let result = user_internal_service
        .get_session(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[post("/get_user_info")]
pub async fn get_user_info(
    body: Bytes,
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<Option<UserCombinedInfoDTO>>, ApplicationErrorBox> {
    let query: UserInfoQuery = parse_request_body(body)?;

    let result = user_internal_service
        .get_user_info(query)
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_user_info")]
pub async fn db_get_user_info(
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<Vec<DbUserDTO>>, ApplicationErrorBox> {
    let result = user_internal_service
        .db_get_user_info()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

#[get("/db_get_personal_info")]
pub async fn db_get_personal_info(
    user_internal_service: web::Data<dyn UserInternalService>,
) -> Result<ApiResponse<Vec<DbPersonalInfo>>, ApplicationErrorBox> {
    let result = user_internal_service
        .db_get_personal_info()
        .await
        .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

    ApiResponse::ok(result)
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_password)
        .service(verify_payment_password)
        .service(set_payment_password)
        .service(clear_wrong_payment_password_tried)
        .service(get_session)
        .service(get_user_info)
        .service(db_get_user_info)
        .service(db_get_personal_info);
}
