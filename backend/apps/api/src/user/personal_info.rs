//! 个人信息API控制器
//!
//! 该模块提供个人信息的查询和修改接口，支持：
//! - 获取当前用户所有个人信息
//! - 创建/更新个人信息
//! - 删除个人信息

use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, get, post, web};
use base::application::commands::personal_info::{PersonalInfoQuery, SetPersonalInfoCommand};
use base::application::service::personal_info::{
    PersonalInfoDTO, PersonalInfoService, SetPersonalInfoDTO,
};

/// 获取个人信息列表
///
/// # 请求
/// - Method: GET
/// - Path: /api/user/personal_info
/// - Cookie: session_id
///
/// # 响应
/// - 200 OK: 返回个人信息列表
/// - 403 Forbidden: 会话无效
#[get("/personal_info")]
pub async fn get_personal_info(
    request: HttpRequest,
    personal_info_service: web::Data<dyn PersonalInfoService>,
) -> Result<ApiResponse<Vec<PersonalInfoDTO>>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let query = PersonalInfoQuery { session_id };

    ApiResponse::ok(personal_info_service.get_personal_info(query).await?)
}

/// 设置个人信息（创建/更新/删除）
///
/// # 请求
/// - Method: POST
/// - Path: /api/user/personal_info
/// - Cookie: session_id
/// - Body: SetPersonalInfoDTO
///
/// # 响应
/// - 200 OK: 操作成功
/// - 403 Forbidden: 会话无效
/// - 13001: 身份证号格式错误
/// - 13002: 该身份证号对应的个人信息不存在，或没有权限设置
#[post("/personal_info")]
pub async fn set_personal_info(
    request: HttpRequest,
    body: Bytes,
    personal_info_service: web::Data<dyn PersonalInfoService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&request)?;

    let dto: SetPersonalInfoDTO = parse_request_body(body)?;

    let command = SetPersonalInfoCommand::from_session_id_and_dto(session_id, dto);

    personal_info_service.set_personal_info(command).await?;

    ApiResponse::ok(())
}
