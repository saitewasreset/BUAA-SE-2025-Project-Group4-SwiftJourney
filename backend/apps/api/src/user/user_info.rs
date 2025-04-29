use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, get, post, web};
use base::application::commands::user_profile::{SetUserProfileCommand, UserProfileQuery};
use base::application::service::user_profile::{
    SetUserProfileDTO, UserProfileDTO, UserProfileService,
};

#[get("/user_info")]
pub async fn get_user_info(
    requests: HttpRequest,
    user_info_service: web::Data<dyn UserProfileService>,
) -> Result<ApiResponse<UserProfileDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let query = UserProfileQuery { session_id };

    ApiResponse::ok(user_info_service.get_profile(query).await?)
}

#[post("/user_info")]
pub async fn set_user_info(
    requests: HttpRequest,
    body: Bytes,
    user_info_service: web::Data<dyn UserProfileService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let set_profile_dto: SetUserProfileDTO = parse_request_body(body)?;

    let command = SetUserProfileCommand::from_session_id_and_dto(session_id, set_profile_dto);

    ApiResponse::ok(user_info_service.set_profile(command).await?)
}
