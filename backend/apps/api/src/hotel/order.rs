use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::Bytes;
use actix_web::{HttpRequest, post, web};
use base::application::{
    ApplicationError, GeneralError, commands::hotel_order::HotelOrderRequestsDTO,
    service::hotel_order::HotelOrderService,
};
use serde::{Deserialize, Serialize};
use shared::{API_SUCCESS_CODE, API_SUCCESS_MESSAGE};
use tracing::error;

/// 用于响应的交易信息
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfoDTO {
    pub transaction_id: String,
    pub amount: f64,
    pub status: String,
}

/// 创建酒店预订订单
///
/// POST /api/hotel/order
///
/// 提交订单后，订单为"未支付"状态，本接口将返回`TransactionInfo`，
/// 需要根据`TransactionInfo`中的信息调用`支付订单`接口进行支付。
/// 支付后订单才会真正被处理。
#[post("/order")]
pub async fn create_hotel_order(
    req: HttpRequest,
    body: Bytes,
    hotel_order_service: web::Data<dyn HotelOrderService>,
) -> Result<ApiResponse<TransactionInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&req)?;
    let hotel_orders: HotelOrderRequestsDTO = parse_request_body(body)?;

    let transaction_result = hotel_order_service
        .process_hotel_orders(session_id, hotel_orders)
        .await
        .map_err(|e| {
            error!("Failed to process hotel orders: {:?}", e);
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

    Ok(ApiResponse {
        code: API_SUCCESS_CODE,
        message: API_SUCCESS_MESSAGE.to_string(),
        data: Some(TransactionInfoDTO {
            transaction_id: transaction_result.transaction_id.to_string(),
            amount: transaction_result.amount,
            status: "unpaid".to_string(),
        }),
    })
}
