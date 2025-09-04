use crate::application::commands::transaction::{
    BalanceQuery, CancelOrderCommand, GenerateDebugTransactionCommand, PayTransactionCommand,
    RechargeCommand, SetPaymentPasswordCommand, TransactionDetailQuery, TransactionQuery,
};
use crate::application::service::transaction::{
    BalanceInfoDTO, TransactionApplicationService, TransactionApplicationServiceError,
    transaction_to_dto,
};
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::transaction::{TransactionService, TransactionServiceError};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use shared::application_error::{ApplicationError, GeneralError, ModeError};
use shared::domain::Identifiable;
use shared::domain::model::transaction::{Transaction, TransactionAmountAbs};
use shared::domain::model::user::UserId;
use shared::internal::order::dto::TransactionInfoDTO;
use shared::internal::user::command::{
    ClearWrongPaymentPasswordTriedCommand, SessionQuery, UserInfoQuery, VerifyPasswordCommand,
    VerifyPaymentPasswordCommand,
};
use shared::internal::user::dto::UserCombinedInfoDTO;
use shared::ports::user::UserPort;
use shared::utils::TimeMeter;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

pub struct TransactionApplicationServiceImpl<T, R, UP>
where
    T: TransactionService,
    R: TransactionRepository,
    UP: UserPort,
{
    debug_mode: bool,
    transaction_service: Arc<T>,
    transaction_repository: Arc<R>,
    user_port: Arc<UP>,
}

impl<T, R, UP> TransactionApplicationServiceImpl<T, R, UP>
where
    T: TransactionService,
    R: TransactionRepository,
    UP: UserPort,
{
    pub fn new(
        debug_mode: bool,
        transaction_service: Arc<T>,
        transaction_repository: Arc<R>,
        user_port: Arc<UP>,
    ) -> Self {
        Self {
            debug_mode,
            transaction_service,
            transaction_repository,
            user_port,
        }
    }
    async fn get_user_id_by_session_id(
        &self,
        session_id: &str,
    ) -> Result<UserId, Box<dyn ApplicationError>> {
        let session = self
            .user_port
            .get_session(SessionQuery {
                session_id: session_id.to_string(),
            })
            .await
            .inspect_err(|e| error!("Failed to get session: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        Ok(UserId::from(session.user_id))
    }

    async fn get_user_dto_by_session_id(
        &self,
        session_id: &str,
    ) -> Result<UserCombinedInfoDTO, Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(session_id).await?;

        let user = self
            .user_port
            .get_user_info(UserInfoQuery {
                user_id: user_id.into(),
            })
            .await
            .inspect_err(|e| error!("Failed to get user info: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        Ok(user)
    }

    async fn verify_user_password(
        &self,
        user: &UserCombinedInfoDTO,
        user_password: String,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let passed = self
            .user_port
            .verify_password(VerifyPasswordCommand {
                user_id: user.user_id,
                raw_password: user_password,
            })
            .await
            .inspect_err(|e| error!("Failed to verify user password: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        if passed {
            Ok(())
        } else {
            Err(
                Box::new(TransactionApplicationServiceError::WrongUserPassword)
                    as Box<dyn ApplicationError>,
            )
        }
    }

    async fn verify_payment_password(
        &self,
        user: &UserCombinedInfoDTO,
        payment_password: String,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let passed = self
            .user_port
            .verify_payment_password(VerifyPaymentPasswordCommand {
                user_id: user.user_id,
                raw_payment_password: payment_password,
            })
            .await
            .inspect_err(|e| error!("Failed to verify user payment password: {:?}", e))
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        if passed {
            Ok(())
        } else {
            Err(
                Box::new(TransactionApplicationServiceError::WrongUserPassword)
                    as Box<dyn ApplicationError>,
            )
        }
    }
}

#[async_trait]
impl<T, R, UP> TransactionApplicationService for TransactionApplicationServiceImpl<T, R, UP>
where
    T: TransactionService,
    R: TransactionRepository,
    UP: UserPort,
{
    #[instrument(skip(self))]
    async fn recharge(&self, command: RechargeCommand) -> Result<(), Box<dyn ApplicationError>> {
        let user_id = self
            .get_user_id_by_session_id(&command.session_id)
            .await
            .inspect_err(|e| {
                error!(
                    "failed to get user id by session id {}: {}",
                    command.session_id, e
                );
            })?;

        self.transaction_service
            .recharge(
                user_id,
                TransactionAmountAbs::from(Decimal::from_f64(command.amount).ok_or(
                    GeneralError::BadRequest(format!("Invalid amount: {}", command.amount)),
                )?),
            )
            .await
            .inspect_err(|e| {
                error!("failed to recharge user {}: {}", user_id, e);
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn query_balance(
        &self,
        query: BalanceQuery,
    ) -> Result<BalanceInfoDTO, Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(&query.session_id).await?;

        let balance = self.transaction_service.get_balance(user_id).await?;

        Ok(BalanceInfoDTO {
            balance: balance.to_f64().unwrap(),
        })
    }

    #[instrument(skip(self))]
    async fn query_transactions(
        &self,
        query: TransactionQuery,
    ) -> Result<Vec<TransactionInfoDTO>, Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(&query.session_id).await?;

        let tx_list = self
            .transaction_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|e| {
                error!("failed to find tx list for user_id {}: {}", user_id, e);
                GeneralError::InternalServerError
            })?;

        Ok(tx_list
            .into_iter()
            .map(transaction_to_dto)
            .collect())
    }

    #[instrument(skip(self, command))]
    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let user = self.get_user_dto_by_session_id(&command.session_id).await?;

        self.verify_user_password(&user, command.user_password)
            .await?;

        self.user_port
            .set_payment_password(shared::internal::user::command::SetPaymentPasswordCommand {
                user_id: user.user_id,
                raw_payment_password: Some(command.payment_password),
            })
            .await
            .map_err(|e| {
                error!(
                    "failed to set payment password for user: {}: {:?}",
                    user.user_id, e
                );
                GeneralError::InternalServerError
            })?;

        self.user_port
            .clear_wrong_payment_password_tried(ClearWrongPaymentPasswordTriedCommand {
                user_id: user.user_id,
            })
            .await
            .map_err(|e| {
                error!(
                    "failed to clear wrong payment password tried times for user: {}: {}",
                    user.user_id, e
                );
                GeneralError::InternalServerError
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn pay_transaction(
        &self,
        command: PayTransactionCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let user = self.get_user_dto_by_session_id(&command.session_id).await?;

        if let Some(user_password) = command.user_password {
            self.verify_user_password(&user, user_password).await?;
        } else if let Some(payment_password) = command.payment_password {
            self.verify_payment_password(&user, payment_password)
                .await?;
        } else {
            return Err(Box::new(GeneralError::BadRequest(
                "Neither user password nor payment password was set".to_string(),
            )));
        }

        self.transaction_service
            .pay_transaction(command.transaction_id)
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn generate_debug_transaction(
        &self,
        command: GenerateDebugTransactionCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        if !self.debug_mode {
            return Err(Box::new(ModeError));
        }

        let user_id = self.get_user_id_by_session_id(&command.session_id).await?;

        let amount = TransactionAmountAbs::from_f64_checked(command.amount)
            .map_err(|e| GeneralError::BadRequest(e.to_string()))?;

        let mut tx = Transaction::new_debug(user_id, amount);

        self.transaction_repository
            .save(&mut tx)
            .await
            .map_err(|e| {
                error!("failed to save transaction: {}", e);

                GeneralError::InternalServerError
            })?;

        Ok(transaction_to_dto(tx))
    }

    #[instrument(skip(self))]
    async fn query_transaction_details(
        &self,
        query: TransactionDetailQuery,
    ) -> Result<Vec<TransactionDataDto>, Box<dyn ApplicationError>> {
        let mut meter = TimeMeter::new("query_transaction_details");

        let user_id = self.get_user_id_by_session_id(&query.session_id).await?;

        let transaction_list = self
            .transaction_repository
            .find_by_user_id(user_id)
            .await
            .inspect_err(|e| error!("failed to find transactions for user_id {}: {}", user_id, e))
            .map_err(|e| {
                error!("failed to query transaction for user_id {}: {}", user_id, e);

                GeneralError::InternalServerError
            })?;

        meter.meter("load transaction list");

        let mut result = Vec::with_capacity(transaction_list.len());

        for transaction in transaction_list {
            let txid = transaction.get_id();
            result.push(
                self.transaction_service
                    .convert_transaction_to_dto(transaction)
                    .await
                    .inspect_err(|e| {
                        error!("failed to convert transaction {:?} to dto: {}", txid, e);
                    })
                    .map_err(|e| {
                        error!("failed to convert transaction {:?} to dto {}", txid, e);

                        GeneralError::InternalServerError
                    })?,
            )
        }

        meter.meter("convert transactions to DTO");

        info!("{}", meter);

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn cancel_order(
        &self,
        command: CancelOrderCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(&command.session_id).await?;

        let tx_list = self
            .transaction_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|e| {
                error!("failed to find tx list for user_id {}: {}", user_id, e);
                GeneralError::InternalServerError
            })?;

        let target_order_uuid = command.order_id;

        let mut target_tx = None;
        let mut target_order = None;

        for tx in &tx_list {
            for order in tx.orders() {
                if order.uuid() == target_order_uuid {
                    target_order = Some(order);
                    target_tx = Some(tx);
                    break;
                }
            }
        }

        let target_tx = target_tx.ok_or_else(|| {
            warn!("No transaction found for order id {}", target_order_uuid);
            GeneralError::NotFound(format!(
                "No transaction found for order id {}",
                target_order_uuid
            ))
        })?;

        let target_order = target_order.unwrap();

        self.transaction_service
            .refund_transaction(target_tx.uuid(), std::slice::from_ref(target_order))
            .await
            .map_err(|e| match e {
                TransactionServiceError::RefundError(e) => Box::new(
                    TransactionApplicationServiceError::RefundError(e.to_string()),
                )
                    as Box<dyn ApplicationError>,
                x => {
                    error!("Failed to refund order {}: {}", target_order_uuid, x);
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                }
            })?;

        Ok(())
    }
}
