use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, get, post, web};
use base::application::GeneralError;
use base::application::commands::hotel::{
    HotelInfoQuery, HotelQuery, NewCommentCommand, QuotaQuery, TargetType,
};
use base::application::service::hotel::{
    HotelCommentQuotaDTO, HotelDetailInfoDTO, HotelGeneralInfoDTO, HotelService, NewHotelCommentDTO,
};
use chrono::NaiveDate;
use serde::Deserialize;
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

#[get("/info/{hotel_id}")]
async fn get_hotel_info(
    hotel_id: web::Path<Uuid>,
    requests: HttpRequest,
    hotel_service: web::Data<dyn HotelService>,
) -> Result<ApiResponse<HotelDetailInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let query = HotelInfoQuery {
        session_id,
        hotel_id: *hotel_id,
    };

    let result = hotel_service.query_hotel_info(query).await?;

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

/// 酒店查询请求体DTO
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HotelQueryDTO {
    /// 目标城市/车站名称
    pub target: String,

    /// 目标类型：city或station
    pub target_type: String,

    /// 搜索关键词（可选）
    pub search: Option<String>,

    /// 入住日期（可选）
    pub begin_date: Option<String>,

    /// 离开日期（可选）
    pub end_date: Option<String>,
}

#[post("/query")]
async fn query_hotels(
    requests: HttpRequest,
    body: Bytes,
    hotel_service: web::Data<dyn HotelService>,
) -> Result<ApiResponse<Vec<HotelGeneralInfoDTO>>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let query_dto: HotelQueryDTO = parse_request_body(body)?;

    let target_type = match query_dto.target_type.as_str() {
        "city" => TargetType::City,
        "station" => TargetType::Station,
        _ => {
            return Err(ApplicationErrorBox(Box::new(GeneralError::BadRequest(
                "Invalid target type".into(),
            ))));
        }
    };

    let begin_date = if let Some(date_str) = query_dto.begin_date {
        Some(
            NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
                Box::new(GeneralError::BadRequest("Invalid begin date format".into()))
                    as Box<dyn base::application::ApplicationError>
            })?,
        )
    } else {
        None
    };

    let end_date = if let Some(date_str) = query_dto.end_date {
        Some(
            NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
                Box::new(GeneralError::BadRequest("Invalid end date format".into()))
                    as Box<dyn base::application::ApplicationError>
            })?,
        )
    } else {
        None
    };

    let command = HotelQuery {
        session_id,
        target: query_dto.target,
        target_type,
        search: query_dto.search,
        begin_date,
        end_date,
    };

    let result = hotel_service.query_hotels(command).await?;

    ApiResponse::ok(result)
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_quota)
        .service(get_hotel_info)
        .service(add_comment)
        .service(query_hotels);
}
