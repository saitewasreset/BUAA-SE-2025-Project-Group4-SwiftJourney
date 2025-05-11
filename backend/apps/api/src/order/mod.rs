use crate::{ApiResponse, ApplicationErrorBox, get_session_id};
use actix_web::web::Data;
use actix_web::{HttpRequest, get, web};
use base::application::commands::transaction::TransactionDetailQuery;
use base::application::service::transaction::TransactionApplicationService;
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

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(query_transaction_details);
}
