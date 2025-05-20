use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::cookie::Cookie;
use actix_web::cookie::time::Duration;
use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, HttpResponse, post};
use base::application::commands::user_manager::{
    UserLoginCommand, UserLogoutCommand, UserRegisterCommand, UserUpdatePasswordCommand,
};
use base::application::service::user_manager::{
    UserLoginDTO, UserManagerService, UserRegisterDTO, UserUpdatePasswordDTO,
};
use shared::{API_SUCCESS_CODE, API_SUCCESS_MESSAGE};

#[post("/register")]
async fn register(
    body: Bytes,
    user_manager_service: Data<dyn UserManagerService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let user_register_dto: UserRegisterDTO = parse_request_body(body)?;

    let command = UserRegisterCommand::from(user_register_dto);

    ApiResponse::ok(user_manager_service.register(command).await?)
}

#[post("/login")]
async fn login(body: Bytes, user_manager_service: Data<dyn UserManagerService>) -> HttpResponse {
    match parse_request_body::<UserLoginDTO>(body) {
        Ok(user_login_dto) => {
            let command = UserLoginCommand::from(user_login_dto);

            match user_manager_service.login(command).await {
                Ok(session_id) => {
                    let cookie = Cookie::build("session_id", session_id.to_string())
                        .path("/")
                        .http_only(true)
                        .max_age(Duration::days(30))
                        .finish();
                    HttpResponse::Ok().cookie(cookie).json(ApiResponse {
                        code: API_SUCCESS_CODE,
                        message: API_SUCCESS_MESSAGE.to_string(),
                        data: Some(()),
                    })
                }
                Err(e) => {
                    let api_response: ApiResponse<()> = ApiResponse {
                        code: e.error_code(),
                        message: e.error_message(),
                        data: None,
                    };

                    HttpResponse::Ok().json(api_response)
                }
            }
        }
        Err(e) => {
            let api_response: ApiResponse<()> = ApiResponse {
                code: e.error_code(),
                message: e.error_message(),
                data: None,
            };

            HttpResponse::Ok().body(serde_json::to_string(&api_response).unwrap())
        }
    }
}

#[post("/logout")]
async fn logout(
    request: HttpRequest,
    user_manager_service: Data<dyn UserManagerService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let command = UserLogoutCommand { session_id };

    ApiResponse::ok(user_manager_service.logout(command).await?)
}

#[post("/update_password")]
async fn update_password(
    request: HttpRequest,
    body: Bytes,
    user_manager_service: Data<dyn UserManagerService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let dto: UserUpdatePasswordDTO = parse_request_body(body)?;

    let command = UserUpdatePasswordCommand {
        session_id,
        origin_password: dto.origin_password,
        new_password: dto.new_password,
    };

    user_manager_service.update_password(command).await?;

    ApiResponse::ok(())
}
