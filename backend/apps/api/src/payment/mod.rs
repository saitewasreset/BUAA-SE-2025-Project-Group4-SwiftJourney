use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, get, post, web};
use base::application::commands::transaction::{
    BalanceQuery, PayTransactionCommand, RechargeCommand, SetPaymentPasswordCommand,
    TransactionQuery,
};
use base::application::service::transaction::{
    BalanceInfoDTO, PaymentConfirmationDTO, PaymentPasswordInfoDTO, RechargeDTO,
    TransactionApplicationService, TransactionInfoDTO,
};
use sea_orm::prelude::Uuid;
use serde::Deserialize;

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

#[get("/balance")]
pub async fn query_balance(
    requests: HttpRequest,
    transaction_service: Data<dyn TransactionApplicationService>,
) -> Result<ApiResponse<BalanceInfoDTO>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let balance_query = BalanceQuery { session_id };

    let balance_info_dto = transaction_service.query_balance(balance_query).await?;

    ApiResponse::ok(balance_info_dto)
}

#[get("/")]
pub async fn query_transactions(
    requests: HttpRequest,
    transaction_service: Data<dyn TransactionApplicationService>,
) -> Result<ApiResponse<Vec<TransactionInfoDTO>>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let transaction_query = TransactionQuery { session_id };

    let transaction_info_list = transaction_service
        .query_transactions(transaction_query)
        .await?;

    ApiResponse::ok(transaction_info_list)
}

#[post("/payment_password")]
pub async fn set_payment_password(
    requests: HttpRequest,
    body: Bytes,
    transaction_service: Data<dyn TransactionApplicationService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let set_payment_password_dto: PaymentPasswordInfoDTO = parse_request_body(body)?;

    let command = SetPaymentPasswordCommand {
        session_id,
        user_password: set_payment_password_dto.user_password,
        payment_password: set_payment_password_dto.payment_password,
    };

    transaction_service.set_payment_password(command).await?;

    ApiResponse::ok(())
}

#[derive(Deserialize)]
struct PayTransactionInfo {
    transaction_id: Uuid,
}

#[post("/pay/{transaction_id}")]
async fn pay_transaction(
    info: web::Path<PayTransactionInfo>,
    requests: HttpRequest,
    body: Bytes,
    transaction_service: Data<dyn TransactionApplicationService>,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let payment_confirmation_dto: PaymentConfirmationDTO = parse_request_body(body)?;

    let command = PayTransactionCommand {
        session_id,
        transaction_id: info.transaction_id,
        user_password: payment_confirmation_dto.user_password,
        payment_password: payment_confirmation_dto.payment_password,
    };

    transaction_service.pay_transaction(command).await?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(recharge)
        .service(query_balance)
        .service(query_transactions)
        .service(set_payment_password)
        .service(pay_transaction);
}
