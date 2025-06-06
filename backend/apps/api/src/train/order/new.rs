use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::{HttpRequest, post, web::Bytes, web::Data};
use base::application::{
    ApplicationError, GeneralError,
    service::train_order::{OrderPacksDTO, TrainOrderService},
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

/// 创建火车票订单
///
/// POST /api/train/order/new
///
/// 提交订单后，订单为"未支付"状态，本接口将返回`TransactionInfo`，
/// 需要根据`TransactionInfo`中的信息调用`支付订单`接口进行支付。
/// 支付后订单才会真正被处理。
#[post("/new")]
pub async fn create_train_order(
    req: HttpRequest,
    body: Bytes,
    train_order_service: Data<dyn TrainOrderService>,
) -> Result<ApiResponse<TransactionInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&req)?;
    let order_packs: OrderPacksDTO = parse_request_body(body)?;

    let transaction_result = train_order_service
        .process_train_order_packs(session_id, order_packs)
        .await?;

    if transaction_result.transaction_id.is_nil() {
        error!("No transaction was created");
        return Err(ApplicationErrorBox::from(
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>,
        ));
    }

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
