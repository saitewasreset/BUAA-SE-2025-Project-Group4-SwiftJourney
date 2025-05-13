use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, get, post, web};
use base::application::commands::personal_info::{PersonalInfoQuery, SetPersonalInfoCommand};
use base::application::service::personal_info::{
    PersonalInfoDTO, PersonalInfoService, SetPersonalInfoDTO,
};

#[get("/personal_info")]
pub async fn get_personal_info(
    requests: HttpRequest,
    personal_info_service: web::Data<dyn PersonalInfoService>,
) -> Result<ApiResponse<PersonalInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let query = PersonalInfoQuery { session_id };

    ApiResponse::ok(personal_info_service.get_personal_info(query).await?)
}

#[post("/personal_info")]
pub async fn set_personal_info(
    requests: HttpRequest,
    body: Bytes,
    personal_info_service: web::Data<dyn PersonalInfoService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let set_personal_info_dto: SetPersonalInfoDTO = parse_request_body(body)?;

    let command = SetPersonalInfoCommand::from_session_id_and_dto(session_id, set_personal_info_dto);

    ApiResponse::ok(personal_info_service.set_personal_info(command).await?)
}