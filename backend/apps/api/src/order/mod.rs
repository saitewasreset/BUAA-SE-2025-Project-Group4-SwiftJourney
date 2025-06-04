use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, get, post, web};
use base::application::commands::transaction::{CancelOrderCommand, TransactionDetailQuery};
use base::application::service::transaction::{CancelOrderDTO, TransactionApplicationService};
use base::domain::service::order::order_dto::TransactionDataDto;

#[get("/list")]
pub async fn query_transaction_details(
    requests: HttpRequest,
    transaction_service: Data<dyn TransactionApplicationService>,
) -> Result<ApiResponse<Vec<TransactionDataDto>>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let transaction_detail_query = TransactionDetailQuery { session_id };

    let transaction_info_list = transaction_service
        .query_transaction_details(transaction_detail_query)
        .await?;

    ApiResponse::ok(transaction_info_list)
}

#[post("/cancel")]
pub async fn cancel_order(
    requests: HttpRequest,
    body: Bytes,
    transaction_service: Data<dyn TransactionApplicationService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let cancel_order_dto: CancelOrderDTO = parse_request_body(body)?;

    let cancel_order_command = CancelOrderCommand {
        session_id,
        order_id: cancel_order_dto.order_id,
    };

    transaction_service
        .cancel_order(cancel_order_command)
        .await?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_transaction_details).service(cancel_order);
}
