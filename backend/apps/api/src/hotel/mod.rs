use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, get, post, web};
use base::application::commands::hotel::{NewCommentCommand, QuotaQuery};
use base::application::service::hotel::{HotelCommentQuotaDTO, HotelService, NewHotelCommentDTO};
use uuid::Uuid;

#[get("/quota/{hotel_id}")]
async fn get_quota(
    hotel_id: web::Path<Uuid>,
    requests: HttpRequest,
    hotel_service: web::Data<dyn HotelService>,
) -> Result<ApiResponse<HotelCommentQuotaDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let query = QuotaQuery {
        session_id,
        hotel_id: *hotel_id,
    };

    let result = hotel_service.get_quota(query).await?;

    ApiResponse::ok(result)
}

#[post("/comment")]
async fn add_comment(
    requests: HttpRequest,
    body: Bytes,
    hotel_service: web::Data<dyn HotelService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let new_hotel_command_dto: NewHotelCommentDTO = parse_request_body(body)?;

    let command = NewCommentCommand {
        session_id,
        hotel_id: new_hotel_command_dto.hotel_id,
        rating: new_hotel_command_dto.rating,
        comment: new_hotel_command_dto.comment,
    };

    hotel_service.new_comment(command).await?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_quota).service(add_comment);
}
