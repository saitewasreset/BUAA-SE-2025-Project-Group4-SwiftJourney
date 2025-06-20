use crate::application::commands::transaction::{
    BalanceQuery, CancelOrderCommand, GenerateDebugTransactionCommand, PayTransactionCommand,
    RechargeCommand, SetPaymentPasswordCommand, TransactionDetailQuery, TransactionQuery,
};
use crate::application::service::transaction::{
    BalanceInfoDTO, TransactionApplicationService, TransactionApplicationServiceError,
    TransactionInfoDTO,
};
use crate::application::{ApplicationError, GeneralError, ModeError};
use crate::domain::Identifiable;
use crate::domain::model::session::SessionId;
use crate::domain::model::transaction::{Transaction, TransactionAmountAbs};
use crate::domain::model::user::{PaymentPassword, User, UserId};
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::repository::user::UserRepository;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::transaction::{TransactionService, TransactionServiceError};
use crate::domain::service::user::{UserService, UserServiceError};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use shared::utils::TimeMeter;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

pub struct TransactionApplicationServiceImpl<S, T, R, U, UR>
where
    S: SessionManagerService,
    T: TransactionService,
    R: TransactionRepository,
    U: UserService,
    UR: UserRepository,
{
    debug_mode: bool,
    session_manager: Arc<S>,
    transaction_service: Arc<T>,
    transaction_repository: Arc<R>,
    user_service: Arc<U>,
    user_repository: Arc<UR>,
}

impl<S, T, R, U, UR> TransactionApplicationServiceImpl<S, T, R, U, UR>
where
    S: SessionManagerService,
    T: TransactionService,
    R: TransactionRepository,
    U: UserService,
    UR: UserRepository,
{
    pub fn new(
        debug_mode: bool,
        session_manager: Arc<S>,
        transaction_service: Arc<T>,
        transaction_repository: Arc<R>,
        user_service: Arc<U>,
        user_repository: Arc<UR>,
    ) -> Self {
        Self {
            debug_mode,
            session_manager,
            transaction_service,
            transaction_repository,
            user_service,
            user_repository,
        }
    }
    async fn get_user_id_by_session_id(
        &self,
        session_id: &str,
    ) -> Result<UserId, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(session_id)
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        Ok(user_id)
    }

    async fn get_user_by_session_id(
        &self,
        session_id: &str,
    ) -> Result<User, Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(session_id).await?;

        let user = self
            .user_repository
            .find(user_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        Ok(user)
    }

    async fn verify_user_password(
        &self,
        user: &User,
        user_password: String,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.user_service
            .verify_password(user, user_password)
            .await
            .map_err(|e| match e {
                UserServiceError::InvalidPassword => {
                    Box::new(TransactionApplicationServiceError::WrongUserPassword)
                        as Box<dyn ApplicationError>
                }
                _ => Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>,
            })?;

        Ok(())
    }

    async fn verify_payment_password(
        &self,
        user: &User,
        payment_password: String,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.user_service
            .verify_payment_password(user, payment_password)
            .await
            .map_err(|e| match e {
                UserServiceError::InvalidPassword => {
                    Box::new(TransactionApplicationServiceError::WrongUserPassword)
                        as Box<dyn ApplicationError>
                }
                _ => Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>,
            })?;

        Ok(())
    }
}

#[async_trait]
impl<S, T, R, U, UR> TransactionApplicationService
    for TransactionApplicationServiceImpl<S, T, R, U, UR>
where
    S: SessionManagerService,
    T: TransactionService,
    R: TransactionRepository,
    U: UserService,
    UR: UserRepository,
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

        Ok(tx_list.into_iter().map(|item| item.into()).collect())
    }

    #[instrument(skip(self, command))]
    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let user = self.get_user_by_session_id(&command.session_id).await?;

        self.verify_user_password(&user, command.user_password)
            .await?;

        let payment_password = PaymentPassword::try_from(command.payment_password.as_str())
            .map_err(|_for_super_earth| {
                TransactionApplicationServiceError::InvalidPaymentPasswordFormat
            })?;

        self.user_service
            .set_payment_password(user.get_id().unwrap(), Some(payment_password))
            .await
            .map_err(|e| {
                error!(
                    "failed to set payment password for user: {}: {}",
                    user.get_id().unwrap(),
                    e
                );
                GeneralError::InternalServerError
            })?;

        self.user_service
            .clear_wrong_payment_password_tried(user.get_id().unwrap())
            .await
            .map_err(|e| {
                error!(
                    "failed to clear wrong payment password tried times for user: {}: {}",
                    user.get_id().unwrap(),
                    e
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
        let user = self.get_user_by_session_id(&command.session_id).await?;

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

        Ok(tx.into())
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
            GeneralError::NotFound
        })?;

        let target_order = target_order.unwrap();

        self.transaction_service
            .refund_transaction(target_tx.uuid(), &[target_order.clone()])
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
