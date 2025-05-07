use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, post, web};
use base::application::commands::transaction::RechargeCommand;
use base::application::service::transaction::{RechargeDTO, TransactionApplicationService};

#[post("/recharge")]
pub async fn recharge(
    requests: HttpRequest,
    body: Bytes,
    transaction_service: Data<dyn TransactionApplicationService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let recharge_info_dto: RechargeDTO = parse_request_body(body)?;

    let recharge_command = RechargeCommand {
        session_id,
        amount: recharge_info_dto.amount,
    };

    transaction_service.recharge(recharge_command).await?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(recharge);
}
